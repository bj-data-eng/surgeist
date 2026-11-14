//! Public native Gaussian pixel oracles. Rational coefficients below follow
//! exp(-k² ln2), independently of the production kernel generator.
use super::convolution::{assert_pixel, bounds, image, pixel, renderer_and_surface};
use crate::{
    Color, FilterColorSpace, FilterDomains, FilterEdgeMode, GaussianInput, GaussianSource, Image,
    ImageBuffer, ImageFit, Layer, Parameters, PreparedGaussian, Rect, RenderAccountingContext,
    Renderer, Scene, Surface, Transform, prepare_gaussian,
};

pub(super) fn operation(x: f64, y: f64, color: FilterColorSpace) -> PreparedGaussian {
    prepare_gaussian(
        GaussianInput::new().with_standard_deviations(x, y),
        color,
        &mut RenderAccountingContext::default(),
    )
    .unwrap()
}

pub(super) fn source_scene(source: Image, destination: Rect) -> Scene {
    let mut scene = Scene::new();
    scene.image(source, destination, ImageFit::Stretch);
    scene
}

pub(super) fn finite_scene(
    source: Scene,
    domains: FilterDomains,
    edge: FilterEdgeMode,
    operation: PreparedGaussian,
    layer: Layer,
) -> Scene {
    let mut scene = Scene::new();
    scene
        .gaussian_layer(
            layer,
            operation,
            GaussianSource::finite_with_edge(source, domains, edge),
        )
        .unwrap();
    scene
}

pub(super) fn render(renderer: &mut Renderer, surface: &mut Surface, scene: &Scene) -> ImageBuffer {
    let stats = pollster::block_on(renderer.render(surface, scene, Parameters::default())).unwrap();
    assert_eq!(stats, renderer.stats());
    pollster::block_on(renderer.read_headless(surface)).unwrap()
}

pub(super) fn sigma() -> f64 {
    1.0 / (2.0 * 2.0_f64.ln()).sqrt()
}

// Center, +/-1, +/-2, +/-3, +/-4. The normalization denominator is
// 65536 + 2*(32768+4096+128+1) =139522.
const WEIGHTS: [f64; 5] = [
    65536.0 / 139522.0,
    32768.0 / 139522.0,
    4096.0 / 139522.0,
    128.0 / 139522.0,
    1.0 / 139522.0,
];
const HALF_WEIGHTS: [f64; 3] = [65536.0 / 73730.0, 4096.0 / 73730.0, 1.0 / 73730.0];

fn byte(value: f64) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}
fn white(alpha: f64) -> [u8; 4] {
    if byte(alpha) == 0 {
        [0; 4]
    } else {
        [255, 255, 255, byte(alpha)]
    }
}

// Test-only extension definition derived directly from the published modes.
fn address(index: i64, len: i64, edge: FilterEdgeMode) -> Option<usize> {
    if (0..len).contains(&index) {
        return Some(index as usize);
    }
    match edge {
        FilterEdgeMode::None => None,
        FilterEdgeMode::Duplicate => Some(index.clamp(0, len - 1) as usize),
        FilterEdgeMode::Wrap => Some(index.rem_euclid(len) as usize),
        FilterEdgeMode::Mirror => {
            let phase = index.rem_euclid(2 * len);
            Some(if phase < len {
                phase
            } else {
                2 * len - 1 - phase
            } as usize)
        }
    }
}
fn axis_oracle(input: &[f64], position: i64, weights: &[f64], edge: FilterEdgeMode) -> f64 {
    let radius = weights.len() as i64 - 1;
    (-radius..=radius)
        .map(|k| {
            address(position + k, input.len() as i64, edge).map_or(0.0, |i| input[i])
                * weights[k.unsigned_abs() as usize]
        })
        .sum()
}

#[test]
fn public_gaussian_rational_anisotropic_impulse_matches_both_working_formats() {
    let (mut renderer, mut surface) = renderer_and_surface(9, 9);
    let domain = bounds(9, 9);
    let source = image(
        9,
        9,
        (0..81)
            .flat_map(|i| if i == 40 { [255; 4] } else { [0; 4] })
            .collect(),
    );
    let scene = finite_scene(
        source_scene(source, domain),
        FilterDomains::try_new(domain, domain, domain).unwrap(),
        FilterEdgeMode::None,
        operation(sigma(), sigma() / 2.0, FilterColorSpace::Srgb),
        Layer::new(),
    );
    for format in [
        crate::resource::WorkingFormat::HighPrecision,
        crate::resource::WorkingFormat::ReducedPrecision,
    ] {
        pollster::block_on(renderer.render_with_exact_graph_working_format_for_test(
            &mut surface,
            &scene,
            Parameters::default(),
            format,
        ))
        .unwrap();
        let output = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        for y in 0..9 {
            for x in 0..9 {
                let dx = (x as i32 - 4).unsigned_abs() as usize;
                let dy = (y as i32 - 4).unsigned_abs() as usize;
                let alpha = if dy < HALF_WEIGHTS.len() {
                    WEIGHTS[dx] * HALF_WEIGHTS[dy]
                } else {
                    0.0
                };
                assert_pixel(&output, x, y, white(alpha), 2);
            }
        }
    }
}

#[test]
fn public_gaussian_all_edges_extend_original_finite_cells_at_distant_indices() {
    let (mut renderer, mut surface) = renderer_and_surface(13, 3);
    let source_domain = Rect::new(5.0, 1.0, 3.0, 1.0);
    let full = bounds(13, 3);
    for edge in [
        FilterEdgeMode::Duplicate,
        FilterEdgeMode::Wrap,
        FilterEdgeMode::Mirror,
        FilterEdgeMode::None,
    ] {
        let source = source_scene(
            image(
                3,
                1,
                vec![255, 255, 255, 64, 255, 255, 255, 128, 255, 255, 255, 192],
            ),
            source_domain,
        );
        let scene = finite_scene(
            source,
            FilterDomains::try_new(source_domain, full, full).unwrap(),
            edge,
            operation(sigma(), 0.0, FilterColorSpace::Srgb),
            Layer::new(),
        );
        let output = render(&mut renderer, &mut surface, &scene);
        for x in 0..13 {
            let alpha = axis_oracle(
                &[64.0 / 255.0, 128.0 / 255.0, 192.0 / 255.0],
                i64::from(x) - 5,
                &WEIGHTS,
                edge,
            );
            assert_pixel(&output, x, 1, white(alpha), 2);
            let outside = if edge == FilterEdgeMode::None {
                0.0
            } else {
                alpha
            };
            assert_pixel(&output, x, 0, white(outside), 2);
        }
    }
}

#[test]
fn public_gaussian_zero_negative_and_signed_zero_preserve_numeric_input_without_extension() {
    let (mut renderer, mut surface) = renderer_and_surface(5, 2);
    let source = Rect::new(1.0, 0.0, 2.0, 1.0);
    let full = bounds(5, 2);
    for (sx, sy) in [(0.0, 0.0), (-0.0, 0.0), (-1.0, 1e300), (1e300, -1.0)] {
        let scene = finite_scene(
            source_scene(
                image(2, 1, vec![31, 117, 219, 128, 200, 19, 73, 255]),
                source,
            ),
            FilterDomains::try_new(source, full, full).unwrap(),
            FilterEdgeMode::Duplicate,
            operation(sx, sy, FilterColorSpace::LinearSrgb),
            Layer::new(),
        );
        let output = render(&mut renderer, &mut surface, &scene);
        assert_pixel(&output, 1, 0, [31, 117, 219, 128], 2);
        assert_pixel(&output, 2, 0, [200, 19, 73, 255], 1);
        for (x, y) in [(0, 0), (3, 0), (1, 1)] {
            assert_pixel(&output, x, y, [0; 4], 1);
        }
        let usage = renderer.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.work(crate::RenderWork::KernelSampleVisits), 0);
    }
}

#[test]
fn public_gaussian_each_one_zero_axis_leaves_the_other_axis_active() {
    let (mut renderer, mut surface) = renderer_and_surface(9, 9);
    let full = bounds(9, 9);
    for (sx, sy) in [(sigma(), 0.0), (0.0, sigma())] {
        let scene = finite_scene(
            source_scene(
                image(
                    9,
                    9,
                    (0..81)
                        .flat_map(|i| if i == 40 { [255; 4] } else { [0; 4] })
                        .collect(),
                ),
                full,
            ),
            FilterDomains::try_new(full, full, full).unwrap(),
            FilterEdgeMode::None,
            operation(sx, sy, FilterColorSpace::Srgb),
            Layer::new(),
        );
        let output = render(&mut renderer, &mut surface, &scene);
        for y in 0..9 {
            for x in 0..9 {
                let (distance, off_axis) = if sx > 0.0 {
                    ((x as i32 - 4).unsigned_abs() as usize, y != 4)
                } else {
                    ((y as i32 - 4).unsigned_abs() as usize, x != 4)
                };
                assert_pixel(
                    &output,
                    x,
                    y,
                    white(if off_axis { 0.0 } else { WEIGHTS[distance] }),
                    2,
                );
            }
        }
    }
}

#[test]
fn public_gaussian_source_and_result_half_cells_have_independent_coverage_and_vertical_halo() {
    let (mut renderer, mut surface) = renderer_and_surface(5, 5);
    let full = bounds(5, 5);
    for (input, filter, output, coverage) in [
        (Rect::new(2.0, 2.0, 1.0, 1.0), full, full, 1.0),
        (Rect::new(2.0, 2.0, 0.5, 1.0), full, full, 0.5),
        (
            Rect::new(2.0, 2.0, 0.5, 1.0),
            full,
            Rect::new(2.5, 1.0, 1.5, 1.0),
            0.25,
        ),
        // F independently halves both source and result cell coverage.
        (
            Rect::new(2.0, 2.0, 1.0, 1.0),
            Rect::new(2.0, 0.0, 0.5, 5.0),
            full,
            0.25,
        ),
    ] {
        let scene = finite_scene(
            source_scene(image(1, 1, vec![255; 4]), Rect::new(2.0, 2.0, 1.0, 1.0)),
            FilterDomains::try_new(input, filter, output).unwrap(),
            FilterEdgeMode::None,
            operation(sigma(), sigma(), FilterColorSpace::Srgb),
            Layer::new(),
        );
        let pixels = render(&mut renderer, &mut surface, &scene);
        // Output row1 needs the original row2, outside P in the third case.
        assert_pixel(&pixels, 2, 1, white(WEIGHTS[0] * WEIGHTS[1] * coverage), 2);
        if output == full {
            assert_pixel(&pixels, 2, 2, white(WEIGHTS[0] * WEIGHTS[0] * coverage), 2);
        } else {
            assert_pixel(&pixels, 2, 2, [0; 4], 1);
        }
    }
}

#[test]
fn public_gaussian_constant_infinite_precedence_avoids_finite_edge_darkening() {
    let (mut renderer, mut surface) = renderer_and_surface(4, 2);
    let filter = bounds(4, 2);
    let result = Rect::new(0.5, 0.0, 3.0, 2.0);
    let color = Color::try_rgba(0.25, 0.5, 0.75, 0.5).unwrap();
    for space in [FilterColorSpace::Srgb, FilterColorSpace::LinearSrgb] {
        for (sx, sy) in [
            (sigma(), sigma() / 2.0),
            (0.0, 0.0),
            (-1.0, 1e300),
            (1e300, 1e300),
        ] {
            let mut scene = Scene::new();
            scene
                .gaussian_layer(
                    Layer::new(),
                    operation(sx, sy, space),
                    GaussianSource::constant(color, filter, result).unwrap(),
                )
                .unwrap();
            let output = render(&mut renderer, &mut surface, &scene);
            assert_pixel(&output, 0, 0, [64, 128, 191, 64], 2);
            assert_pixel(&output, 1, 0, [64, 128, 191, 128], 2);
            assert_pixel(&output, 3, 1, [64, 128, 191, 64], 2);
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
    let scene = finite_scene(
        source_scene(image(4, 2, [64, 128, 191, 128].repeat(8)), filter),
        FilterDomains::try_new(filter, filter, filter).unwrap(),
        FilterEdgeMode::None,
        operation(sigma(), sigma(), FilterColorSpace::Srgb),
        Layer::new(),
    );
    let output = render(&mut renderer, &mut surface, &scene);
    assert!(
        pixel(&output, 0, 0)[3] < 100,
        "finite None must darken near F"
    );
}

#[test]
fn public_gaussian_fractional_period_wraps_actual_texel_canvas_without_a_global_grid_seam() {
    let (mut renderer, mut surface) = renderer_and_surface(13, 2);
    let period = Rect::new(0.0, 0.0, 2.5, 1.25);
    let filter = Rect::new(-5.0, 0.0, 13.0, 1.25);
    for active in [false, true] {
        let mut tile = Scene::new();
        tile.fill(
            Rect::new(2.5 / 3.0, 0.0, 2.5 / 3.0, 1.25),
            Color::try_rgba(1.0, 1.0, 1.0, 1.0).unwrap(),
        );
        let mut scene = Scene::new();
        scene
            .gaussian_layer(
                Layer::new().with_transform(Transform::translation(5.0, 0.0).unwrap()),
                operation(
                    if active { sigma() * 2.5 / 3.0 } else { 0.0 },
                    0.0,
                    FilterColorSpace::Srgb,
                ),
                GaussianSource::periodic(tile, period, filter, filter).unwrap(),
            )
            .unwrap();
        let output = render(&mut renderer, &mut surface, &scene);
        let cells = if active {
            (0..3)
                .map(|x| axis_oracle(&[0.0, 1.0, 0.0], x, &WEIGHTS, FilterEdgeMode::Wrap))
                .collect::<Vec<_>>()
        } else {
            vec![0.0, 1.0, 0.0]
        };
        // hx=5/6: x=.5 maps to texel .1, as do x=5.5 and-4.5.
        let expected = byte(cells[0] * 0.9 + cells[1] * 0.1);
        for x in [0, 5, 10] {
            assert_pixel(&output, x, 0, [255, 255, 255, expected], 3);
        }
    }
}

#[test]
fn public_gaussian_periodic_scope_restores_nonunit_root_and_following_siblings() {
    use crate::{OffsetInput, Options, Size, prepare_offset};
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(10.0, 5.0), 1.5)).unwrap();
    let mut scene = Scene::new();
    let first_period = Rect::new(0.0, 0.0, 2.5, 1.4);
    let mut tile = Scene::new();
    // Nx4,Ny3 under root rho1.5. Nested content covers exactly column1.
    tile.layer(
        Layer::new().with_transform(Transform::translation(0.625, 0.0).unwrap()),
        |child| {
            child.fill(
                Rect::new(0.0, 0.0, 0.625, 1.4),
                Color::try_rgba(1.0, 1.0, 1.0, 1.0).unwrap(),
            );
        },
    );
    scene
        .gaussian_layer(
            Layer::new(),
            operation(0.0, 0.0, FilterColorSpace::Srgb),
            GaussianSource::periodic(tile, first_period, first_period, first_period).unwrap(),
        )
        .unwrap();
    scene.layer(Layer::new(), |child| {
        child.fill(
            Rect::new(3.0, 0.0, 2.0, 1.0),
            Color::try_rgba(0.0, 1.0, 0.0, 1.0).unwrap(),
        );
    });
    let sibling = Rect::new(3.0, 2.0, 2.0, 1.0);
    let offset = prepare_offset(
        OffsetInput::new(),
        FilterDomains::try_new(sibling, sibling, sibling).unwrap(),
        &mut RenderAccountingContext::default(),
    )
    .unwrap();
    scene.layer(Layer::new().with_offset(offset), |child| {
        child.fill(sibling, Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap());
    });
    let second_period = Rect::new(0.3, 0.2, 1.4, 2.5);
    let second_output = Rect::new(0.0, 0.0, 2.5, 3.0);
    let mut second = Scene::new();
    // Nx3,Ny4; at p.x1, texel coordinate is(1-.3)*3/1.4-.5=1.
    second.fill(
        Rect::new(0.3 + 1.4 / 3.0, 0.2, 1.4 / 3.0, 2.5),
        Color::try_rgba(1.0, 1.0, 1.0, 1.0).unwrap(),
    );
    scene
        .gaussian_layer(
            Layer::new().with_transform(Transform::translation(6.0, 0.0).unwrap()),
            operation(0.0, 0.0, FilterColorSpace::Srgb),
            GaussianSource::periodic(second, second_period, second_output, second_output).unwrap(),
        )
        .unwrap();
    let output = render(&mut renderer, &mut surface, &scene);
    // First p.x1 ->1.1 in its own4-cellperiod, 90% white.
    assert_pixel(&output, 1, 0, [255, 255, 255, 230], 3);
    assert_pixel(&output, 5, 0, [0, 255, 0, 255], 2);
    assert_pixel(&output, 5, 3, [255, 0, 0, 255], 2);
    assert_pixel(&output, 10, 0, [255; 4], 3);
    assert_pixel(&output, 14, 7, [0; 4], 1);
}

#[test]
fn public_gaussian_two_axes_and_multibatch_keep_one_filter_color_conversion() {
    let (mut renderer, mut surface) = renderer_and_surface(3, 3);
    let full = bounds(3, 3);
    let s = sigma();
    for (space, linear) in [
        (FilterColorSpace::Srgb, false),
        (FilterColorSpace::LinearSrgb, true),
    ] {
        let row = [[255, 0, 0, 128], [0, 0, 255, 255], [255, 0, 0, 128]];
        let scene = finite_scene(
            source_scene(image(3, 3, row.repeat(3).concat()), full),
            FilterDomains::try_new(full, full, full).unwrap(),
            FilterEdgeMode::Wrap,
            operation(s, s / 2.0, space),
            Layer::new(),
        );
        let output = render(&mut renderer, &mut surface, &scene);
        let blue = axis_oracle(&[0.0, 1.0, 0.0], 1, &WEIGHTS, FilterEdgeMode::Wrap);
        let red = (1.0 - blue) * (128.0 / 255.0);
        let alpha = red + blue;
        let encode = |v: f64| {
            if !linear {
                v
            } else if v <= 0.0031308 {
                12.92 * v
            } else {
                1.055 * v.powf(1.0 / 2.4) - 0.055
            }
        };
        assert_pixel(
            &output,
            1,
            1,
            [
                byte(encode(red / alpha)),
                0,
                byte(encode(blue / alpha)),
                byte(alpha),
            ],
            3,
        );
    }
    // sigma4 gives R16/33records. A scalar independent exp(-k²/32)
    // distribution over the alternating periodic source reveals lost carry.
    for space in [FilterColorSpace::Srgb, FilterColorSpace::LinearSrgb] {
        let source = source_scene(
            image(
                3,
                3,
                [[255; 4], [0, 0, 0, 255], [255; 4]].repeat(3).concat(),
            ),
            full,
        );
        let scene = finite_scene(
            source,
            FilterDomains::try_new(full, full, full).unwrap(),
            FilterEdgeMode::Wrap,
            operation(4.0, 4.0, space),
            Layer::new(),
        );
        let output = render(&mut renderer, &mut surface, &scene);
        let mut norm = 0.0;
        let mut light = 0.0;
        for k in -16i32..=16 {
            let w = (-f64::from(k * k) / 32.0).exp();
            norm += w;
            if (1 + k).rem_euclid(3) != 1 {
                light += w;
            }
        }
        let v = light / norm;
        let expected = if space == FilterColorSpace::LinearSrgb {
            1.055 * v.powf(1.0 / 2.4) - 0.055
        } else {
            v
        };
        assert_pixel(
            &output,
            1,
            1,
            [byte(expected), byte(expected), byte(expected), 255],
            3,
        );
    }
}

#[test]
fn public_gaussian_empty_finite_input_clears_only_its_nonpreserving_output_footprint() {
    use crate::{BlendMode, LayerComposition, PorterDuffOperator};
    let (mut renderer, mut surface) = renderer_and_surface(5, 1);
    let full = bounds(5, 1);
    for source in [
        Rect::new(1.0, 0.0, 0.0, 1e300),
        Rect::new(1.0, 0.0, 2.0, 1.0),
    ] {
        let output = Rect::new(1.0, 0.0, 2.0, 1.0);
        let mut scene = Scene::new();
        scene.fill(full, Color::try_rgba(0.0, 1.0, 0.0, 1.0).unwrap());
        scene
            .gaussian_layer(
                Layer::new().with_composition(
                    LayerComposition::try_porter_duff(PorterDuffOperator::Copy, BlendMode::Normal)
                        .unwrap(),
                ),
                operation(1e300, 1e300, FilterColorSpace::LinearSrgb),
                GaussianSource::finite(
                    Scene::new(),
                    FilterDomains::try_new(source, full, output).unwrap(),
                ),
            )
            .unwrap();
        let pixels = render(&mut renderer, &mut surface, &scene);
        for x in 0..5 {
            assert_pixel(
                &pixels,
                x,
                0,
                if (1..3).contains(&x) {
                    [0; 4]
                } else {
                    [0, 255, 0, 255]
                },
                1,
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
fn public_gaussian_large_logical_origin_preserves_neighbor_distinctions() {
    let (mut renderer, mut surface) = renderer_and_surface(3, 1);
    let domain = Rect::new(100_000_000.0, 0.0, 3.0, 1.0);
    let source = source_scene(
        image(3, 1, vec![255, 0, 0, 255, 0, 0, 255, 255, 255, 0, 0, 255]),
        domain,
    );
    let scene = finite_scene(
        source,
        FilterDomains::try_new(domain, domain, domain).unwrap(),
        FilterEdgeMode::Wrap,
        operation(sigma(), 0.0, FilterColorSpace::Srgb),
        Layer::new().with_transform(Transform::translation(-100_000_000.0, 0.0).unwrap()),
    );
    let output = render(&mut renderer, &mut surface, &scene);
    let blue = axis_oracle(&[0.0, 1.0, 0.0], 1, &WEIGHTS, FilterEdgeMode::Wrap);
    assert_pixel(&output, 1, 0, [byte(1.0 - blue), 0, byte(blue), 255], 3);
    assert!(pixel(&output, 0, 0)[0] > pixel(&output, 1, 0)[0]);
}

#[test]
fn public_gaussian_full_outer_affines_preserve_local_vector_domains() {
    let (mut renderer, mut surface) = renderer_and_surface(18, 14);
    let domain = Rect::new(1.0, 1.0, 3.0, 3.0);
    // For the skew, rho=sqrt((9.25+sqrt(9.25²-4*3.5²))/2).
    // D uses device indices[2,12), with first/last coverage .23443556/.06225775.
    // Rectangle raster coverage then independent D image coverage gives their
    // squares. Normalized exp(-k²/(2*(sigma*rho)²)) with duplicate endpoints,
    // separable filtering and bilinear reconstruction at A^-1(8.5,7.5)
    // gives alpha .4476513503, hence114. Other two grids have full cells.
    for (affine, inside, outside, alpha) in [
        ([-1.0, 0.0, 0.0, 1.0, 8.0, 0.0], (5, 2), (2, 2), 128),
        ([0.0, 1.0, -1.0, 0.0, 8.0, 0.0], (5, 2), (2, 2), 128),
        ([2.0, 0.5, 1.0, 2.0, 0.0, 0.0], (8, 7), (1, 1), 114),
    ] {
        let mut source = Scene::new();
        source.layer(Layer::new().try_opacity(0.5).unwrap(), |nested| {
            nested.fill(domain, Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap());
        });
        let scene = finite_scene(
            source,
            FilterDomains::try_new(domain, domain, domain).unwrap(),
            FilterEdgeMode::Duplicate,
            operation(sigma(), sigma() / 2.0, FilterColorSpace::Srgb),
            Layer::new().with_transform(Transform::try_new(affine).unwrap()),
        );
        let output = render(&mut renderer, &mut surface, &scene);
        assert_pixel(&output, inside.0, inside.1, [255, 0, 0, alpha], 3);
        assert_pixel(&output, outside.0, outside.1, [0; 4], 1);
    }
}

#[test]
fn public_gaussian_outer_clip_mask_and_opacity_stay_after_filtering() {
    use crate::{ResolvedLayerAlphaMask, Shape};
    let (mut renderer, mut surface) = renderer_and_surface(6, 1);
    let full = bounds(6, 1);
    let mask = ResolvedLayerAlphaMask::try_new(
        image(
            6,
            1,
            [255_u8, 255, 128, 0, 255, 255]
                .into_iter()
                .flat_map(|a| [255, 255, 255, a])
                .collect(),
        ),
        full,
    )
    .unwrap();
    let layer = Layer::new()
        .try_clip(Shape::rect(Rect::new(2.0, 0.0, 2.0, 1.0)))
        .unwrap()
        .with_resolved_alpha_mask(mask)
        .try_opacity(0.5)
        .unwrap();
    let source = source_scene(image(6, 1, [255, 0, 0, 255].repeat(6)), full);
    let scene = finite_scene(
        source,
        FilterDomains::try_new(full, full, full).unwrap(),
        FilterEdgeMode::Duplicate,
        operation(sigma(), sigma() / 2.0, FilterColorSpace::Srgb),
        layer,
    );
    let output = render(&mut renderer, &mut surface, &scene);
    assert_pixel(&output, 2, 0, [255, 0, 0, 64], 2);
    for x in [0, 1, 3, 4, 5] {
        assert_pixel(&output, x, 0, [0; 4], 1);
    }
}

#[test]
fn public_gaussian_backdrop_only_source_restores_original_finite_lattice() {
    use crate::{BackdropCaptureBounds, BackdropFilterInput, FilterBlur, FilterList, FilterOp};
    let (mut renderer, mut surface) = renderer_and_surface(6, 1);
    let full = bounds(6, 1);
    let source = Rect::new(1.0, 0.0, 2.0, 1.0);
    let backdrop = BackdropFilterInput::try_new(
        FilterList::try_ops(vec![FilterOp::blur(FilterBlur::try_new(0.0).unwrap())]).unwrap(),
        BackdropCaptureBounds::try_new(full).unwrap(),
        None,
    )
    .unwrap();
    let mut scene = Scene::new();
    scene.fill(
        Rect::new(1.0, 0.0, 1.0, 1.0),
        Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap(),
    );
    scene
        .gaussian_layer(
            Layer::new().try_backdrop_filter(backdrop).unwrap(),
            operation(sigma(), 0.0, FilterColorSpace::Srgb),
            GaussianSource::finite_with_edge(
                Scene::new(),
                FilterDomains::try_new(source, full, source).unwrap(),
                FilterEdgeMode::Duplicate,
            ),
        )
        .unwrap();
    let output = render(&mut renderer, &mut surface, &scene);
    // Finite D has[opaque red,transparent]. At its second cell Duplicate
    // selects red for k<=-1, giving one half of the non-center mass.
    let expected = (1.0 - WEIGHTS[0]) / 2.0;
    assert_pixel(&output, 2, 0, [255, 0, 0, byte(expected)], 2);
    assert_pixel(&output, 3, 0, [0; 4], 1);
}

#[test]
fn public_gaussian_empty_input_or_output_excludes_unused_backdrop_and_kernel_work() {
    use crate::{
        BackdropCaptureBounds, BackdropFilterInput, FilterList, FilterOp, UnitFilterAmount,
    };
    let (mut renderer, mut surface) = renderer_and_surface(3, 1);
    let full = bounds(3, 1);
    let empty = Rect::new(0.0, 0.0, 0.0, 1e300);
    for (source, result) in [(empty, full), (full, empty)] {
        let backdrop = BackdropFilterInput::try_new(
            FilterList::try_ops(vec![FilterOp::invert(
                UnitFilterAmount::try_new(1.0).unwrap(),
            )])
            .unwrap(),
            BackdropCaptureBounds::try_new(Rect::new(0.0, 0.0, 1e12, 1e12)).unwrap(),
            None,
        )
        .unwrap();
        let mut scene = Scene::new();
        scene.fill(full, Color::try_rgba(0.0, 1.0, 0.0, 1.0).unwrap());
        let input = source_scene(image(3, 1, [255; 4].repeat(3)), full);
        scene
            .gaussian_layer(
                Layer::new().try_backdrop_filter(backdrop).unwrap(),
                operation(1e300, 1e300, FilterColorSpace::LinearSrgb),
                GaussianSource::finite(
                    input,
                    FilterDomains::try_new(source, full, result).unwrap(),
                ),
            )
            .unwrap();
        let pixels = render(&mut renderer, &mut surface, &scene);
        for x in 0..3 {
            assert_pixel(&pixels, x, 0, [0, 255, 0, 255], 1);
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
fn public_gaussian_periodic_two_axis_kernels_use_distinct_actual_densities() {
    let (mut renderer, mut surface) = renderer_and_surface(8, 3);
    let period = Rect::new(0.25, 0.375, 2.5, 1.25);
    let output = bounds(8, 3);
    let tile = source_scene(
        image(
            3,
            2,
            vec![
                0, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
        ),
        period,
    );
    let mut scene = Scene::new();
    scene
        .gaussian_layer(
            Layer::new(),
            operation(
                sigma() * 2.5 / 3.0,
                sigma() / 2.0 * 1.25 / 2.0,
                FilterColorSpace::Srgb,
            ),
            GaussianSource::periodic(tile, period, output, output).unwrap(),
        )
        .unwrap();
    let pixels = render(&mut renderer, &mut surface, &scene);
    let horizontal = (0..3)
        .map(|x| axis_oracle(&[0.0, 1.0, 0.0], x, &WEIGHTS, FilterEdgeMode::Wrap))
        .collect::<Vec<_>>();
    let vertical = (0..2)
        .map(|y| axis_oracle(&[1.0, 0.0], y, &HALF_WEIGHTS, FilterEdgeMode::Wrap))
        .collect::<Vec<_>>();
    for y in 0..3 {
        for x in 0..8 {
            let qx = ((f64::from(x) + 0.5 - 0.25) * 1.2 - 0.5).rem_euclid(3.0);
            let qy = ((f64::from(y) + 0.5 - 0.375) * 1.6 - 0.5).rem_euclid(2.0);
            let ix = qx.floor() as usize;
            let iy = qy.floor() as usize;
            let vx = horizontal[ix] * (1.0 - qx.fract()) + horizontal[(ix + 1) % 3] * qx.fract();
            let vy = vertical[iy] * (1.0 - qy.fract()) + vertical[(iy + 1) % 2] * qy.fract();
            assert_pixel(&pixels, x, y, white(vx * vy), 3);
        }
    }
}

#[test]
fn public_gaussian_reachable_huge_sigma_refuses_atomically_and_valid_scene_retries() {
    let (mut renderer, mut surface) = renderer_and_surface(3, 1);
    let full = bounds(3, 1);
    let build = |sigma_x| {
        finite_scene(
            source_scene(image(3, 1, [255, 0, 0, 255].repeat(3)), full),
            FilterDomains::try_new(full, full, full).unwrap(),
            FilterEdgeMode::Duplicate,
            operation(sigma_x, 0.0, FilterColorSpace::Srgb),
            Layer::new(),
        )
    };
    let valid = build(sigma());
    let before = render(&mut renderer, &mut surface, &valid);
    let stats = renderer.stats();
    let error =
        pollster::block_on(renderer.render(&mut surface, &build(1e300), Parameters::default()))
            .unwrap_err();
    assert!(
        matches!(
            error.gaussian_diagnostic(),
            Some(crate::GaussianPreparationError::Numeric {
                phase: crate::GaussianNumericPhase::KernelExtent,
                ..
            })
        ),
        "{error:?}"
    );
    assert_eq!(renderer.stats(), stats);
    assert_eq!(
        pollster::block_on(renderer.read_headless(&surface)).unwrap(),
        before
    );
    assert_eq!(render(&mut renderer, &mut surface, &valid), before);
}

#[test]
fn public_gaussian_fractional_output_bounds_destructive_operators_without_double_coverage() {
    use crate::{
        BlendMode, LayerComposition, PorterDuffOperator as Pd, ResolvedLayerAlphaMask, Shape,
    };
    let (mut renderer, mut surface) = renderer_and_surface(4, 1);
    let full = bounds(4, 1);
    let output = Rect::new(0.5, 0.0, 2.0, 1.0);
    let parent: [f64; 4] = [0.0, 0.25, 0.0, 1.0];
    for format in [
        crate::resource::WorkingFormat::HighPrecision,
        crate::resource::WorkingFormat::ReducedPrecision,
    ] {
        for operator in [
            Pd::Copy,
            Pd::SourceIn,
            Pd::SourceOut,
            Pd::DestinationIn,
            Pd::DestinationAtop,
            Pd::Clear,
            Pd::SourceOver,
        ] {
            let mask = ResolvedLayerAlphaMask::try_new(
                image(
                    4,
                    1,
                    [255_u8, 128, 64, 255]
                        .into_iter()
                        .flat_map(|a| [255, 255, 255, a])
                        .collect(),
                ),
                full,
            )
            .unwrap();
            let layer = Layer::new()
                .with_composition(
                    LayerComposition::try_porter_duff(operator, BlendMode::Normal).unwrap(),
                )
                .try_opacity(0.5)
                .unwrap()
                .with_resolved_alpha_mask(mask)
                .try_clip(Shape::rect(Rect::new(0.0, 0.0, 2.25, 1.0)))
                .unwrap();
            let mut scene = Scene::new();
            scene.fill(full, Color::try_rgba(0.0, 0.25, 0.0, 1.0).unwrap());
            scene
                .gaussian_layer(
                    layer,
                    operation(sigma(), sigma(), FilterColorSpace::LinearSrgb),
                    GaussianSource::constant(
                        Color::try_rgba(0.8, 0.2, 0.0, 0.5).unwrap(),
                        full,
                        output,
                    )
                    .unwrap(),
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
            for x in 0..4 {
                let c = [0.5, 1.0, 0.5, 0.0][x as usize];
                let mask = [1.0, 128.0 / 255.0, 64.0 / 255.0, 1.0][x as usize];
                let source_a = 0.5 * 0.5 * mask;
                let source = [0.8 * source_a, 0.2 * source_a, 0.0, source_a];
                let (fs, fd) = match operator {
                    Pd::Copy => (1.0, 0.0),
                    Pd::SourceIn => (1.0, 0.0),
                    Pd::SourceOut => (0.0, 0.0),
                    Pd::DestinationIn => (0.0, source_a),
                    Pd::DestinationAtop => (0.0, source_a),
                    Pd::Clear => (0.0, 0.0),
                    Pd::SourceOver => (1.0, 1.0 - source_a),
                    _ => unreachable!(),
                };
                let author = [1.0, 1.0, 0.25, 0.0][x as usize];
                // Independently evaluate the full-coverage operation first,
                // then mix its result by O coverage, then the authored clip.
                let complete = std::array::from_fn::<_, 4, _>(|i| fs * source[i] + fd * parent[i]);
                let premul = std::array::from_fn::<_, 4, _>(|i| {
                    author * (c * complete[i] + (1.0 - c) * parent[i]) + (1.0 - author) * parent[i]
                });
                let alpha = premul[3].clamp(0.0, 1.0);
                let actual = pixel(&pixels, x, 0);
                let actual_alpha = f64::from(actual[3]) / 255.0;
                // Source/parent storage, whole-operation clip, destination
                // storage and straight readback each round. For this bounded
                // fixture their propagated premul/alpha error is <=2.5byte
                // levels; a low-alpha straight RGB division is not that bound.
                assert!(
                    (actual_alpha - alpha).abs() * 255.0 <= 2.5,
                    "alpha at {x}: {actual:?}, ideal {premul:?}"
                );
                for channel in 0..3 {
                    let reconstructed = f64::from(actual[channel]) / 255.0 * actual_alpha;
                    assert!(
                        (reconstructed - premul[channel]).abs() * 255.0 <= 2.5,
                        "premul channel{channel} at{x}: {actual:?}, ideal{premul:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn public_gaussian_fractional_empty_copy_preserves_exterior_and_singular_copy_is_empty() {
    use crate::{BlendMode, LayerComposition, PorterDuffOperator};
    let (mut renderer, mut surface) = renderer_and_surface(4, 1);
    let full = bounds(4, 1);
    let output = Rect::new(0.5, 0.0, 2.0, 1.0);
    for singular in [false, true] {
        let layer = Layer::new().with_composition(
            LayerComposition::try_porter_duff(PorterDuffOperator::Copy, BlendMode::Normal).unwrap(),
        );
        let layer = if singular {
            layer.with_transform(Transform::try_new([1.0, 0.0, 0.0, 0.0, 0.0, 0.0]).unwrap())
        } else {
            layer
        };
        let mut scene = Scene::new();
        scene.fill(full, Color::try_rgba(0.0, 1.0, 0.0, 1.0).unwrap());
        scene
            .gaussian_layer(
                layer,
                operation(1e300, 1e300, FilterColorSpace::Srgb),
                GaussianSource::finite(
                    Scene::new(),
                    FilterDomains::try_new(full, full, output).unwrap(),
                ),
            )
            .unwrap();
        let pixels = render(&mut renderer, &mut surface, &scene);
        for x in 0..4 {
            let a = if singular {
                255
            } else {
                [128, 0, 128, 255][x as usize]
            };
            assert_pixel(
                &pixels,
                x,
                0,
                if a == 0 { [0; 4] } else { [0, 255, 0, a] },
                2,
            );
        }
    }
}

#[test]
fn public_gaussian_active_float32_refusal_keeps_constant_and_identity_routes_executable() {
    use crate::{RuntimeCapabilityUnavailableReason, RuntimeOperation};
    let (mut renderer, mut surface) = renderer_and_surface(3, 1);
    let full = bounds(3, 1);
    let available = renderer
        .default_device_capabilities_for_test()
        .effect_precisions();
    assert!(available.supports_filter_float32());
    assert!(renderer.override_default_device_filter_float32_fact_for_test(false));
    assert!(
        !renderer
            .default_device_capabilities_for_test()
            .effect_precisions()
            .supports_filter_float32()
    );
    let mut constant = Scene::new();
    constant
        .gaussian_layer(
            Layer::new(),
            operation(1e300, 1e300, FilterColorSpace::Srgb),
            GaussianSource::constant(Color::BLACK, full, full).unwrap(),
        )
        .unwrap();
    let pixels = render(&mut renderer, &mut surface, &constant);
    assert_pixel(&pixels, 1, 0, [0, 0, 0, 255], 1);
    let build = |s| {
        finite_scene(
            source_scene(image(3, 1, [255, 0, 0, 255].repeat(3)), full),
            FilterDomains::try_new(full, full, full).unwrap(),
            FilterEdgeMode::Duplicate,
            operation(s, 0.0, FilterColorSpace::Srgb),
            Layer::new(),
        )
    };
    let before = render(&mut renderer, &mut surface, &build(0.0));
    let stats = renderer.stats();
    let error =
        pollster::block_on(renderer.render(&mut surface, &build(sigma()), Parameters::default()))
            .unwrap_err();
    let diagnostic = error.runtime_capability_unavailable_diagnostic().unwrap();
    assert_eq!(diagnostic.operation(), RuntimeOperation::GaussianRendering);
    assert_eq!(
        diagnostic.reason(),
        RuntimeCapabilityUnavailableReason::FilterFloat32Unavailable
    );
    assert_eq!(renderer.stats(), stats);
    assert_eq!(
        pollster::block_on(renderer.read_headless(&surface)).unwrap(),
        before
    );
    assert!(
        renderer.override_default_device_filter_float32_fact_for_test(
            available.supports_filter_float32()
        )
    );
    assert_eq!(render(&mut renderer, &mut surface, &build(sigma())), before);
}

#[test]
fn public_gaussian_plus_replaces_copy_and_uses_source_attenuation_without_pd_correction() {
    use crate::{BlendMode, LayerComposition, PorterDuffOperator};
    let (mut renderer, mut surface) = renderer_and_surface(4, 1);
    let full = bounds(4, 1);
    let output = Rect::new(0.5, 0.0, 2.0, 1.0);
    let mut scene = Scene::new();
    scene.fill(full, Color::try_rgba(0.0, 0.25, 0.0, 1.0).unwrap());
    scene
        .gaussian_layer(
            Layer::new()
                .with_composition(
                    LayerComposition::try_porter_duff(PorterDuffOperator::Copy, BlendMode::Normal)
                        .unwrap(),
                )
                .blend(BlendMode::Plus)
                .try_opacity(0.5)
                .unwrap(),
            operation(sigma(), sigma(), FilterColorSpace::Srgb),
            GaussianSource::constant(Color::try_rgba(0.8, 0.2, 0.0, 0.5).unwrap(), full, output)
                .unwrap(),
        )
        .unwrap();
    let pixels = render(&mut renderer, &mut surface, &scene);
    for (x, c) in [0.5, 1.0, 0.5, 0.0].into_iter().enumerate() {
        // Effective Plus adds the already attenuated source to the parent.
        assert_pixel(
            &pixels,
            x as u32,
            0,
            [byte(0.2 * c), byte(0.25 + 0.05 * c), 0, 255],
            2,
        );
    }
}

#[test]
fn public_gaussian_fractional_copy_reconstructs_coverage_through_the_full_affine() {
    use crate::{BlendMode, LayerComposition, PorterDuffOperator};
    let (mut renderer, mut surface) = renderer_and_surface(10, 7);
    let full = bounds(10, 7);
    let result = Rect::new(0.25, 0.25, 3.0, 2.0);
    let affine = [1.0, 0.25, 0.5, 1.0, 2.0, 1.0];
    let determinant: f64 = 0.875;
    let trace: f64 = 2.3125;
    let rho = ((trace + (trace * trace - 4.0 * determinant * determinant).sqrt()) / 2.0).sqrt();
    let extents = [(3.25 * rho).ceil() as i32, (2.25 * rho).ceil() as i32];
    let cell = |x: i32, y: i32| {
        let x = f64::from(x.clamp(0, extents[0] - 1));
        let y = f64::from(y.clamp(0, extents[1] - 1));
        ((x + 1.0).min(3.25 * rho) - x.max(0.25 * rho)).clamp(0.0, 1.0)
            * ((y + 1.0).min(2.25 * rho) - y.max(0.25 * rho)).clamp(0.0, 1.0)
    };
    let mut scene = Scene::new();
    scene.fill(full, Color::try_rgba(0.0, 0.5, 0.0, 1.0).unwrap());
    scene
        .gaussian_layer(
            Layer::new()
                .with_transform(Transform::try_new(affine).unwrap())
                .with_composition(
                    LayerComposition::try_porter_duff(PorterDuffOperator::Copy, BlendMode::Normal)
                        .unwrap(),
                ),
            operation(sigma(), sigma(), FilterColorSpace::LinearSrgb),
            GaussianSource::constant(Color::BLACK, result, result).unwrap(),
        )
        .unwrap();
    for format in [
        crate::resource::WorkingFormat::HighPrecision,
        crate::resource::WorkingFormat::ReducedPrecision,
    ] {
        pollster::block_on(renderer.render_with_exact_graph_working_format_for_test(
            &mut surface,
            &scene,
            Parameters::default(),
            format,
        ))
        .unwrap();
        let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        for y in 0..7 {
            for x in 0..10 {
                let px = f64::from(x) + 0.5 - 2.0;
                let py = f64::from(y) + 0.5 - 1.0;
                let texel = [
                    (px - 0.5 * py) / determinant * rho,
                    (-0.25 * px + py) / determinant * rho,
                ];
                let c = if texel[0] < 0.0
                    || texel[1] < 0.0
                    || texel[0] >= f64::from(extents[0])
                    || texel[1] >= f64::from(extents[1])
                {
                    0.0
                } else {
                    let q = [texel[0] - 0.5, texel[1] - 0.5];
                    let base = [q[0].floor() as i32, q[1].floor() as i32];
                    let f = [q[0] - q[0].floor(), q[1] - q[1].floor()];
                    let top =
                        cell(base[0], base[1]) * (1.0 - f[0]) + cell(base[0] + 1, base[1]) * f[0];
                    let bottom = cell(base[0], base[1] + 1) * (1.0 - f[0])
                        + cell(base[0] + 1, base[1] + 1) * f[0];
                    top * (1.0 - f[1]) + bottom * f[1]
                };
                assert_pixel(&pixels, x, y, [0, byte(0.5 * (1.0 - c)), 0, 255], 3);
            }
        }
    }
}
