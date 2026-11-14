//! Public native shadow oracles: alpha filtering, finite displacement, flood-in,
//! unchanged original over shadow, then one output coverage.
use super::convolution::{assert_pixel, bounds, image, pixel, renderer_and_surface};
use crate::{
    Color, DropShadowInput, DropShadowSource, FilterColorSpace, FilterDomains, Image, ImageBuffer,
    ImageFit, ImageRendering, Layer, Parameters, PreparedDropShadow, Rect, RenderAccountingContext,
    Renderer, Scene, Surface, prepare_drop_shadow,
};

pub(super) fn operation(input: DropShadowInput, space: FilterColorSpace) -> PreparedDropShadow {
    prepare_drop_shadow(input, space, &mut RenderAccountingContext::default()).unwrap()
}
pub(super) fn source_scene(source: Image, domain: Rect) -> Scene {
    let mut scene = Scene::new();
    scene.image(source, domain, ImageFit::Stretch);
    scene
}
pub(super) fn finite_scene(
    source: Scene,
    domains: FilterDomains,
    operation: PreparedDropShadow,
    layer: Layer,
) -> Scene {
    let mut scene = Scene::new();
    scene
        .drop_shadow_layer(layer, operation, DropShadowSource::finite(source, domains))
        .unwrap();
    scene
}
pub(super) fn render(renderer: &mut Renderer, surface: &mut Surface, scene: &Scene) -> ImageBuffer {
    let stats = pollster::block_on(renderer.render(surface, scene, Parameters::default())).unwrap();
    assert_eq!(stats, renderer.stats());
    assert_eq!(stats.route, Some(crate::RenderRoute::GpuGraph));
    pollster::block_on(renderer.read_headless(surface)).unwrap()
}
pub(super) fn sigma() -> f64 {
    1.0 / (2.0 * 2.0_f64.ln()).sqrt()
}
fn byte(value: f64) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}
fn red(alpha: f64) -> [u8; 4] {
    if byte(alpha) == 0 {
        [0; 4]
    } else {
        [255, 0, 0, byte(alpha)]
    }
}
fn zero_blur() -> DropShadowInput {
    DropShadowInput::new()
        .with_dx(0.0)
        .with_dy(0.0)
        .with_standard_deviation(0.0)
}
const W: [f64; 5] = [
    65536.0 / 139522.0,
    32768.0 / 139522.0,
    4096.0 / 139522.0,
    128.0 / 139522.0,
    1.0 / 139522.0,
];
const HALF_W: [f64; 3] = [65536.0 / 73730.0, 4096.0 / 73730.0, 1.0 / 73730.0];

#[test]
fn public_native_shadow_default_two_offsets_and_sigma_preserve_original_and_cast_shadow() {
    let (mut renderer, mut surface) = renderer_and_surface(9, 9);
    let full = bounds(9, 9);
    let source = source_scene(
        image(
            9,
            9,
            (0..81)
                .flat_map(|i| if i == 30 { [255; 4] } else { [0; 4] })
                .collect(),
        ),
        full,
    );
    let scene = finite_scene(
        source,
        FilterDomains::try_new(full, full, full).unwrap(),
        operation(DropShadowInput::default(), FilterColorSpace::Srgb),
        Layer::new(),
    );
    let output = render(&mut renderer, &mut surface, &scene);
    assert_pixel(&output, 3, 3, [255; 4], 1);
    // Default sigma2 has support8 and normalized center1/Z. dx=dy2
    // places its center at5,5; the original branch is empty there.
    let z: f64 = (-8i32..=8).map(|k| (-f64::from(k * k) / 8.0).exp()).sum();
    assert_pixel(&output, 5, 5, [0, 0, 0, byte(1.0 / (z * z))], 1);
}

#[test]
fn public_native_shadow_merge_order_alpha_and_color_space_match_independent_equations() {
    let (mut renderer, mut surface) = renderer_and_surface(3, 1);
    let full = bounds(3, 1);
    for format in [
        crate::resource::WorkingFormat::HighPrecision,
        crate::resource::WorkingFormat::ReducedPrecision,
    ] {
        for (space, expected) in [
            (FilterColorSpace::Srgb, [51, 0, 204, 159]),
            (FilterColorSpace::LinearSrgb, [124, 0, 231, 159]),
        ] {
            let input = zero_blur().with_flood_color(Color::try_rgba(1.0, 0.0, 0.0, 0.5).unwrap());
            let scene = finite_scene(
                source_scene(image(3, 1, [0, 0, 255, 128].repeat(3)), full),
                FilterDomains::try_new(full, full, full).unwrap(),
                operation(input, space),
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
            // I=(0,0,.5,.5), h=.5*.5=.25; original-over-shadow gives
            // premul(.125,0,.5),a=.625. Linear RGB encodes straight(.2,0,.8).
            assert_pixel(&pixels, 1, 0, expected, 3);
        }
    }
    let scene = finite_scene(
        source_scene(image(3, 1, [0, 0, 255, 128].repeat(3)), full),
        FilterDomains::try_new(full, full, full).unwrap(),
        operation(
            zero_blur()
                .with_flood_color(Color::try_rgba(1.0, 0.0, 0.0, 0.5).unwrap())
                .with_flood_opacity(0.5),
            FilterColorSpace::Srgb,
        ),
        Layer::new(),
    );
    let pixels = render(&mut renderer, &mut surface, &scene);
    // af=.5*.5=.25 exactly once: premul(.0625,0,.5),a=.5625.
    assert_pixel(&pixels, 1, 0, [28, 0, 227, 143], 3);
}

#[test]
fn public_native_shadow_zero_and_negative_sigma_omit_only_blur() {
    let (mut renderer, mut surface) = renderer_and_surface(5, 1);
    let full = bounds(5, 1);
    for (sx, sy) in [(0.0, 0.0), (-0.0, 0.0), (-1.0, 1e300), (1e300, -1.0)] {
        let scene = finite_scene(
            source_scene(
                image(1, 1, [0, 0, 255, 128].to_vec()),
                Rect::new(0.0, 0.0, 1.0, 1.0),
            ),
            FilterDomains::try_new(full, full, full).unwrap(),
            operation(
                zero_blur()
                    .with_dx(2.0)
                    .with_standard_deviations(sx, sy)
                    .with_flood_color(Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap()),
                FilterColorSpace::Srgb,
            ),
            Layer::new(),
        );
        let pixels = render(&mut renderer, &mut surface, &scene);
        assert_pixel(&pixels, 0, 0, [0, 0, 255, 128], 2);
        assert_pixel(&pixels, 2, 0, [255, 0, 0, 128], 2);
        assert_pixel(&pixels, 1, 0, [0; 4], 1);
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
fn public_native_shadow_alpha_only_blur_keeps_original_rgb_sharp_after_multibatch_filtering() {
    let (mut renderer, mut surface) = renderer_and_surface(9, 1);
    let full = bounds(9, 1);
    let mut shadows = Vec::new();
    for colors in [
        [[255, 0, 0, 255], [0, 0, 255, 255]],
        [[0, 255, 0, 255], [255, 255, 255, 255]],
    ] {
        let source = source_scene(image(2, 1, colors.concat()), Rect::new(0.0, 0.0, 2.0, 1.0));
        let scene = finite_scene(
            source,
            FilterDomains::try_new(full, full, full).unwrap(),
            operation(
                zero_blur()
                    .with_dx(4.0)
                    .with_standard_deviations(4.0, 0.0)
                    .with_flood_color(Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap()),
                FilterColorSpace::LinearSrgb,
            ),
            Layer::new(),
        );
        let pixels = render(&mut renderer, &mut surface, &scene);
        assert_pixel(&pixels, 0, 0, colors[0], 2);
        assert_pixel(&pixels, 1, 0, colors[1], 2);
        shadows.push((2..9).map(|x| pixel(&pixels, x, 0)).collect::<Vec<_>>());
    }
    assert_eq!(
        shadows[0], shadows[1],
        "equal alpha must cast identical unobscured shadows despite source RGB"
    );
}

#[test]
fn public_native_shadow_anisotropic_rational_alpha_weights_match_both_formats() {
    let (mut renderer, mut surface) = renderer_and_surface(11, 11);
    let full = bounds(11, 11);
    let scene = finite_scene(
        source_scene(
            image(
                11,
                11,
                (0..121)
                    .flat_map(|i| if i == 48 { [255; 4] } else { [0; 4] })
                    .collect(),
            ),
            full,
        ),
        FilterDomains::try_new(full, full, full).unwrap(),
        operation(
            DropShadowInput::new()
                .with_standard_deviations(sigma(), sigma() / 2.0)
                .with_flood_color(Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap()),
            FilterColorSpace::Srgb,
        ),
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
        let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        for (x, y, alpha) in [
            (6, 6, W[0] * HALF_W[0]),
            (7, 6, W[1] * HALF_W[0]),
            (6, 7, W[0] * HALF_W[1]),
            (8, 6, W[2] * HALF_W[0]),
        ] {
            assert_pixel(&pixels, x, y, red(alpha), 2);
        }
        assert_pixel(&pixels, 4, 4, [255; 4], 1);
    }
}

#[test]
fn public_native_shadow_internal_gaussian_uses_filter_region_before_offset_not_outer_result() {
    let (mut renderer, mut surface) = renderer_and_surface(4, 1);
    let full = bounds(4, 1);
    let result = Rect::new(2.0, 0.0, 1.0, 1.0);
    let scene = finite_scene(
        source_scene(
            image(1, 1, [255; 4].to_vec()),
            Rect::new(0.0, 0.0, 1.0, 1.0),
        ),
        FilterDomains::try_new(full, full, result).unwrap(),
        operation(
            zero_blur()
                .with_dx(2.0)
                .with_flood_color(Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap()),
            FilterColorSpace::Srgb,
        ),
        Layer::new(),
    );
    let pixels = render(&mut renderer, &mut surface, &scene);
    assert_pixel(&pixels, 2, 0, [255, 0, 0, 255], 1);
    assert_pixel(&pixels, 0, 0, [0; 4], 1);
    let filter = bounds(3, 1);
    let scene = finite_scene(
        source_scene(
            image(1, 1, [255; 4].to_vec()),
            Rect::new(1.0, 0.0, 1.0, 1.0),
        ),
        FilterDomains::try_new(filter, filter, filter).unwrap(),
        operation(
            zero_blur()
                .with_dx(1.0)
                .with_standard_deviations(sigma(), 0.0)
                .with_flood_color(Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap()),
            FilterColorSpace::Srgb,
        ),
        Layer::new(),
    );
    let pixels = render(&mut renderer, &mut surface, &scene);
    assert_pixel(&pixels, 0, 0, [0; 4], 1);
    assert_pixel(&pixels, 2, 0, red(W[0]), 2);
}

#[test]
fn public_native_shadow_fractional_infinite_phase_policy_matches_point_171875() {
    let (mut renderer, mut surface) = renderer_and_surface(2, 1);
    let half = Rect::new(0.0, 0.0, 0.5, 1.0);
    for format in [
        crate::resource::WorkingFormat::HighPrecision,
        crate::resource::WorkingFormat::ReducedPrecision,
    ] {
        let mut scene = Scene::new();
        scene
            .drop_shadow_layer(
                Layer::new(),
                operation(
                    zero_blur().with_flood_color(Color::try_rgba(1.0, 0.0, 0.0, 0.5).unwrap()),
                    FilterColorSpace::Srgb,
                ),
                DropShadowSource::constant(
                    Color::try_rgba(0.0, 0.0, 1.0, 0.5).unwrap(),
                    half,
                    half,
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
        // AlphaF=.25; originalF=.25; h=.125; mergea=.34375;
        // final O half gives premul(.046875,0,.125),a=.171875.
        let actual = pixel(&pixels, 0, 0);
        let alpha = f64::from(actual[3]) / 255.0;
        assert!((alpha - 0.171875).abs() * 255.0 <= 2.0, "{actual:?}");
        for (channel, ideal) in [(0, 0.046875), (1, 0.0), (2, 0.125)] {
            assert!(
                (f64::from(actual[channel]) / 255.0 * alpha - ideal).abs() * 255.0 <= 2.0,
                "{actual:?}"
            );
        }
        assert_pixel(&pixels, 1, 0, [0; 4], 1);
    }
}

#[test]
fn public_native_shadow_constant_gaussian_still_has_finite_filter_before_offset() {
    let (mut renderer, mut surface) = renderer_and_surface(3, 1);
    let filter = bounds(2, 1);
    for sx in [0.0, sigma(), 1e300] {
        let mut scene = Scene::new();
        scene
            .drop_shadow_layer(
                Layer::new(),
                operation(
                    zero_blur()
                        .with_dx(1.0)
                        .with_standard_deviations(sx, 0.0)
                        .with_flood_color(Color::try_rgba(1.0, 0.0, 0.0, 0.5).unwrap()),
                    FilterColorSpace::Srgb,
                ),
                DropShadowSource::constant(
                    Color::try_rgba(0.0, 0.0, 1.0, 0.5).unwrap(),
                    filter,
                    filter,
                )
                .unwrap(),
            )
            .unwrap();
        let pixels = render(&mut renderer, &mut surface, &scene);
        assert_pixel(&pixels, 0, 0, [0, 0, 255, 128], 2);
        assert_pixel(&pixels, 1, 0, [51, 0, 204, 159], 3);
        assert_pixel(&pixels, 2, 0, [0; 4], 1);
    }
}

#[test]
fn public_native_shadow_each_reconstruction_hint_uses_existing_covered_finite_alpha_policy() {
    let (mut renderer, mut surface) = renderer_and_surface(7, 1);
    let full = bounds(7, 1);
    for hint in [
        ImageRendering::Auto,
        ImageRendering::Smooth,
        ImageRendering::Pixelated,
        ImageRendering::CrispEdges,
        ImageRendering::HighQuality,
    ] {
        for dx in [1.5, -1.5] {
            let source_x = if dx > 0.0 { 1.0 } else { 3.0 };
            let source = source_scene(
                image(1, 1, [255; 4].to_vec()),
                Rect::new(source_x, 0.0, 1.0, 1.0),
            );
            let scene = finite_scene(
                source,
                FilterDomains::try_new(full, full, full).unwrap(),
                operation(
                    zero_blur()
                        .with_dx(dx)
                        .with_rendering(hint)
                        .with_flood_color(Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap()),
                    FilterColorSpace::Srgb,
                ),
                Layer::new(),
            );
            let pixels = render(&mut renderer, &mut surface, &scene);
            let left = if dx > 0.0 { 2 } else { 1 };
            let expected = match hint {
                ImageRendering::CrispEdges => [1.0, 0.0],
                // One-row transparent exterior: w(.5)=77/144 in X and
                // w(0)=8/9 in Y, rather than a vertically constant stripe.
                ImageRendering::HighQuality => [77.0 / 162.0; 2],
                _ => [0.5; 2],
            };
            for (x, a) in [(left, expected[0]), (left + 1, expected[1])] {
                assert_pixel(&pixels, x, 0, red(a), 2);
            }
        }
    }
}

#[test]
fn public_native_shadow_fractional_finite_source_offset_opacity_and_output_coverage_are_distinct() {
    let (mut renderer, mut surface) = renderer_and_surface(3, 1);
    let filter = bounds(3, 1);
    let source = Rect::new(0.0, 0.0, 0.5, 1.0);
    let output = Rect::new(1.0, 0.0, 0.5, 1.0);
    // The raw image covers a complete cell. D retains half of its alpha .5.
    let scene = finite_scene(
        source_scene(image(1, 1, vec![0, 0, 255, 128]), bounds(1, 1)),
        FilterDomains::try_new(source, filter, output).unwrap(),
        operation(
            zero_blur()
                .with_dx(0.5)
                .with_flood_color(Color::try_rgba(1.0, 0.0, 0.0, 0.5).unwrap())
                .with_flood_opacity(0.5),
            FilterColorSpace::Srgb,
        ),
        Layer::new(),
    );
    let pixels = render(&mut renderer, &mut surface, &scene);
    // .5 raw alpha * .5 D * .5 offset weight * .5 flood * .5 opacity
    // * .5 O = .015625. Original at output cell1 is transparent.
    assert_pixel(&pixels, 1, 0, [255, 0, 0, 4], 1);
    assert_pixel(&pixels, 0, 0, [0; 4], 1);
}

#[test]
fn public_native_shadow_periodic_alpha_becomes_finite_before_displacement() {
    let (mut renderer, mut surface) = renderer_and_surface(5, 1);
    let period = bounds(2, 1);
    let filter = bounds(4, 1);
    for sx in [0.0, sigma()] {
        let tile = source_scene(image(2, 1, vec![0, 0, 255, 128, 0, 0, 0, 0]), period);
        let mut scene = Scene::new();
        scene
            .drop_shadow_layer(
                Layer::new(),
                operation(
                    zero_blur()
                        .with_dx(2.0)
                        .with_standard_deviations(sx, 0.0)
                        .with_flood_color(Color::try_rgba(1.0, 0.0, 0.0, 0.5).unwrap()),
                    FilterColorSpace::Srgb,
                ),
                DropShadowSource::periodic(tile, period, filter, filter).unwrap(),
            )
            .unwrap();
        let pixels = render(&mut renderer, &mut surface, &scene);
        // Unshifted original exists at0, but the finite AlphaF has no cell-2.
        assert_pixel(&pixels, 0, 0, [0, 0, 255, 128], 2);
        assert_pixel(&pixels, 1, 0, [0; 4], 1);
        let alpha_at_zero = if sx == 0.0 {
            0.5
        } else {
            0.5 * (W[0] + 2.0 * W[2] + 2.0 * W[4])
        };
        let shadow = alpha_at_zero * 0.5;
        let a = 0.5 + shadow * 0.5;
        assert_pixel(
            &pixels,
            2,
            0,
            [byte(shadow * 0.5 / a), 0, byte(0.5 / a), byte(a)],
            3,
        );
        assert_pixel(&pixels, 4, 0, [0; 4], 1);
    }
}

#[test]
fn public_native_shadow_fractional_period_uses_one_canvas_and_restores_following_contexts() {
    use crate::{Options, Size, Transform};
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(10.0, 5.0), 1.5)).unwrap();
    let period = Rect::new(0.0, 0.0, 2.5, 1.4);
    let mut tile = Scene::new();
    tile.layer(
        Layer::new().with_transform(Transform::translation(0.625, 0.0).unwrap()),
        |child| {
            child.fill(
                Rect::new(0.0, 0.0, 0.625, 1.4),
                Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap(),
            );
        },
    );
    let mut scene = Scene::new();
    // Transparent flood selects original-period reconstruction without shadow-only work.
    scene
        .drop_shadow_layer(
            Layer::new(),
            operation(zero_blur().with_flood_opacity(0.0), FilterColorSpace::Srgb),
            DropShadowSource::periodic(tile, period, period, period).unwrap(),
        )
        .unwrap();
    scene.layer(Layer::new(), |child| {
        child.fill(
            Rect::new(3.0, 0.0, 2.0, 1.0),
            Color::try_rgba(0.0, 1.0, 0.0, 1.0).unwrap(),
        );
    });
    let second_period = Rect::new(0.3, 0.2, 1.4, 2.5);
    let output = Rect::new(0.0, 0.0, 2.5, 3.0);
    let mut second = Scene::new();
    second.fill(
        Rect::new(0.3 + 1.4 / 3.0, 0.2, 1.4 / 3.0, 2.5),
        Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap(),
    );
    scene
        .drop_shadow_layer(
            Layer::new().with_transform(Transform::translation(6.0, 0.0).unwrap()),
            operation(zero_blur().with_flood_opacity(0.0), FilterColorSpace::Srgb),
            DropShadowSource::periodic(second, second_period, output, output).unwrap(),
        )
        .unwrap();
    let pixels = render(&mut renderer, &mut surface, &scene);
    // First Nx4: local x1 is texel1.1, hence90% opaque blue.
    assert_pixel(&pixels, 1, 0, [0, 0, 255, 230], 3);
    assert_pixel(&pixels, 5, 0, [0, 255, 0, 255], 2);
    // Second Nx3 at local x1 is center of texel1, independently of first B map.
    assert_pixel(&pixels, 10, 0, [255, 0, 0, 255], 3);
    assert_pixel(&pixels, 14, 7, [0; 4], 1);
}

#[test]
fn public_native_shadow_backdrop_capture_is_shared_and_rebased_to_finite_source() {
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
        Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap(),
    );
    scene
        .drop_shadow_layer(
            Layer::new().try_backdrop_filter(backdrop).unwrap(),
            operation(
                zero_blur()
                    .with_dx(1.0)
                    .with_flood_color(Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap()),
                FilterColorSpace::Srgb,
            ),
            DropShadowSource::finite(
                Scene::new(),
                FilterDomains::try_new(source, full, full).unwrap(),
            ),
        )
        .unwrap();
    let pixels = render(&mut renderer, &mut surface, &scene);
    assert_pixel(&pixels, 1, 0, [0, 0, 255, 255], 1);
    assert_pixel(&pixels, 2, 0, [255, 0, 0, 255], 1);
    assert_pixel(&pixels, 3, 0, [0; 4], 1);
}

#[test]
fn public_native_shadow_empty_domains_exclude_unused_backdrop_kernel_and_displacement() {
    use crate::{
        BackdropCaptureBounds, BackdropFilterInput, FilterList, FilterOp, UnitFilterAmount,
    };
    let (mut renderer, mut surface) = renderer_and_surface(3, 1);
    let full = bounds(3, 1);
    let empty = Rect::new(0.0, 0.0, 0.0, 1e300);
    for (source, output) in [(empty, full), (full, empty)] {
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
        scene
            .drop_shadow_layer(
                Layer::new().try_backdrop_filter(backdrop).unwrap(),
                operation(
                    zero_blur()
                        .with_dx(1e300)
                        .with_standard_deviations(1e300, 1e300),
                    FilterColorSpace::LinearSrgb,
                ),
                DropShadowSource::finite(
                    source_scene(image(3, 1, [255; 4].repeat(3)), full),
                    FilterDomains::try_new(source, full, output).unwrap(),
                ),
            )
            .unwrap();
        let stats =
            pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default()))
                .unwrap();
        assert_eq!(stats, renderer.stats());
        // Empty O removes the effect before graph dispatch; an empty D with
        // nonempty O retains a bounded transparent merge. Both preserve parent.
        let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        for x in 0..3 {
            assert_pixel(&pixels, x, 0, [0, 255, 0, 255], 1);
        }
        let usage = renderer.last_attempt().unwrap().attempt().unwrap().usage();
        for work in [
            crate::RenderWork::KernelSampleVisits,
            crate::RenderWork::ScheduledGaussianSamples,
            crate::RenderWork::ScheduledOffsetSamples,
            crate::RenderWork::ScheduledShadowSamples,
        ] {
            assert_eq!(usage.work(work), 0);
        }
    }
}

#[test]
fn public_native_shadow_merge_sample_opportunities_follow_actual_original_branch() {
    let (mut renderer, mut surface) = renderer_and_surface(3, 1);
    let full = bounds(3, 1);
    for periodic in [false, true] {
        for shadow in [false, true] {
            let source = if periodic {
                DropShadowSource::periodic(
                    source_scene(image(2, 1, [0, 0, 255, 128].repeat(2)), bounds(2, 1)),
                    bounds(2, 1),
                    full,
                    full,
                )
                .unwrap()
            } else {
                DropShadowSource::constant(Color::try_rgba(0.0, 0.0, 1.0, 0.5).unwrap(), full, full)
                    .unwrap()
            };
            let mut scene = Scene::new();
            scene
                .drop_shadow_layer(
                    Layer::new(),
                    operation(
                        zero_blur().with_flood_opacity(if shadow { 1.0 } else { 0.0 }),
                        FilterColorSpace::Srgb,
                    ),
                    source,
                )
                .unwrap();
            render(&mut renderer, &mut surface, &scene);
            // Three output cells: authentic constant original has no texture
            // reads; periodic reconstruction has four. Shifted alpha adds one.
            let original_samples = if periodic { 4 } else { 0 };
            let expected = 3 * (original_samples + u64::from(shadow));
            assert_eq!(
                renderer
                    .last_attempt()
                    .unwrap()
                    .attempt()
                    .unwrap()
                    .usage()
                    .work(crate::RenderWork::ScheduledShadowSamples),
                expected,
                "periodic={periodic},shadow={shadow}"
            );
        }
    }
}

#[test]
fn public_native_shadow_nested_vector_full_skew_keeps_original_and_local_displacement_separate() {
    use crate::Transform;
    let (mut renderer, mut surface) = renderer_and_surface(24, 24);
    let full = bounds(9, 7);
    let mut source = Scene::new();
    source.layer(Layer::new().try_opacity(0.5).unwrap(), |child| {
        child.fill(
            Rect::new(2.0, 2.0, 2.0, 2.0),
            Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap(),
        );
    });
    let scene = finite_scene(
        source,
        FilterDomains::try_new(full, full, full).unwrap(),
        operation(
            zero_blur()
                .with_dx(3.0)
                .with_flood_color(Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap()),
            FilterColorSpace::Srgb,
        ),
        Layer::new().with_transform(Transform::try_new([2.0, 1.0, 1.0, 2.0, 0.0, 0.0]).unwrap()),
    );
    let pixels = render(&mut renderer, &mut surface, &scene);
    // A maps original interior(3,3) to(9,9), displaced interior(6,3)
    // to(15,12). Both inverse pixel centers remain well inside their fields.
    assert_pixel(&pixels, 9, 9, [0, 0, 255, 128], 3);
    assert_pixel(&pixels, 15, 12, [255, 0, 0, 128], 3);
    assert_pixel(&pixels, 2, 20, [0; 4], 1);
}

#[test]
fn public_native_shadow_zero_x_axis_preserves_vertical_rational_kernel() {
    let (mut renderer, mut surface) = renderer_and_surface(9, 9);
    let full = bounds(9, 9);
    let scene = finite_scene(
        source_scene(
            image(
                9,
                9,
                (0..81)
                    .flat_map(|i| if i == 30 { [255; 4] } else { [0; 4] })
                    .collect(),
            ),
            full,
        ),
        FilterDomains::try_new(full, full, full).unwrap(),
        operation(
            DropShadowInput::new()
                .with_standard_deviations(0.0, sigma())
                .with_flood_color(Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap()),
            FilterColorSpace::Srgb,
        ),
        Layer::new(),
    );
    let pixels = render(&mut renderer, &mut surface, &scene);
    assert_pixel(&pixels, 3, 3, [255; 4], 1);
    assert_pixel(&pixels, 5, 5, red(W[0]), 2);
    assert_pixel(&pixels, 5, 6, red(W[1]), 2);
    assert_pixel(&pixels, 6, 5, [0; 4], 1);
}

#[test]
fn public_native_shadow_active_fractional_period_filters_alpha_and_reconstructs_unshifted_original()
{
    let (mut renderer, mut surface) = renderer_and_surface(9, 2);
    let period = Rect::new(0.0, 0.0, 2.5, 1.25);
    let filter = Rect::new(0.0, 0.0, 8.0, 1.25);
    let mut tile = Scene::new();
    tile.fill(
        Rect::new(2.5 / 3.0, 0.0, 2.5 / 3.0, 1.25),
        Color::try_rgba(0.0, 0.0, 1.0, 0.5).unwrap(),
    );
    let mut scene = Scene::new();
    scene
        .drop_shadow_layer(
            Layer::new(),
            operation(
                zero_blur()
                    .with_dx(2.5)
                    .with_standard_deviations(sigma() * 2.5 / 3.0, 0.0)
                    .with_flood_color(Color::try_rgba(1.0, 0.0, 0.0, 0.5).unwrap()),
                FilterColorSpace::Srgb,
            ),
            DropShadowSource::periodic(tile, period, filter, filter).unwrap(),
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
        // Nx3, hx5/6. Original p.x5.5 has wrapped texel coordinate.1.
        // Raw alpha cells are[0,.5,0]. Congruent taps on the3-cell period
        // give these independently normalized rational sums.
        let c0 = 0.5 * (W[1] + W[2] + W[4]);
        let c1 = 0.5 * (W[0] + 2.0 * W[3]);
        // Gaussian first resolves to finite unit-grid AlphaF. Its centers
        // 2.5 and3.5 have periodic coordinates2.5 and3.7, hence values
        // c0 and(.3*c0+.7*c1). Offset then averages those cells at3.0.
        // Directly evaluating the periodic function at3.0 would bypass a
        // real reconstruction stage and incorrectly use.9*c0+.1*c1.
        let b = 0.65 * c0 + 0.35 * c1;
        let original = 0.05;
        let h = b * 0.5;
        let a = original + h * (1.0 - original);
        assert_pixel(
            &pixels,
            5,
            0,
            [
                byte(h * (1.0 - original) / a),
                0,
                byte(original / a),
                byte(a),
            ],
            3,
        );
        // The left edge has the same raw period but no displaced AlphaF
        // input at-2: periodic extension does not survive the F resolve.
        assert_pixel(&pixels, 0, 0, [0, 0, 255, byte(original)], 2);
    }
}
