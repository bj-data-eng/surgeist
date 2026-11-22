//! Native turbulence through public scene admission and real GPU readback.

use super::convolution::{assert_pixel, bounds, renderer_and_surface};
use super::displacement::FORMATS;
use super::native_shadow::render;
use crate::{
    accounting::{
        RenderAccountingOptions, RenderAttemptStatus, RenderLimits, RenderQuantity, RenderWork,
    },
    effect::{FilterColorSpace, Turbulence, TurbulenceKind, TurbulenceRegion, TurbulenceStitch},
    geometry::{Rect, Size, Transform},
    layer::{LayerComposition, PorterDuffOperator},
    paint::Color,
    renderer::{Options, Renderer},
    scene::{Layer, Scene},
    stats::RenderRoute,
    style::filter::{BackdropCaptureBounds, BackdropFilterInput, FilterBlur, FilterList, FilterOp},
    surface::Parameters,
};

const FREQUENCY: [f64; 2] = [0.125, 0.2];

fn scene(operation: Turbulence, layer: Layer, filter: Rect, primitive: Rect) -> Scene {
    let mut scene = Scene::new();
    scene
        .turbulence_layer(
            layer,
            operation,
            TurbulenceRegion::try_new(filter, primitive).unwrap(),
        )
        .unwrap();
    scene
}

#[test]
fn seeded_octaves_generate_independent_red_green_blue_and_alpha() {
    let (mut renderer, mut surface) = renderer_and_surface(4, 4);
    let full = bounds(4, 4);
    use TurbulenceKind::{FractalNoise, Turbulence as AbsoluteNoise};

    // Filter Effects 1 §9.21, evaluated independently with integer modular
    // multiplication for Park–Miller and binary64 interpolation at (2.5, 3.5).
    // The inputs 0.99, 7.9 and -7.9 initialize the RNG to 1, 7 and 8.
    for (seed, octaves, kind, expected) in [
        (0.99, 1, FractalNoise, [144, 127, 160, 75]),
        (1.0, 1, AbsoluteNoise, [32, 0, 65, 105]),
        (1.0, 3, FractalNoise, [165, 114, 180, 90]),
        (7.9, 3, FractalNoise, [154, 96, 112, 154]),
        (7.9, 3, AbsoluteNoise, [60, 63, 54, 71]),
        (-7.9, 3, FractalNoise, [177, 171, 105, 125]),
    ] {
        let operation = Turbulence::try_new(FREQUENCY, seed)
            .unwrap()
            .with_octaves(octaves)
            .with_kind(kind)
            .with_color_space(FilterColorSpace::Srgb);
        let scene = scene(operation, Layer::new(), full, full);
        for format in FORMATS {
            let stats =
                pollster::block_on(renderer.render_with_exact_graph_working_format_for_test(
                    &mut surface,
                    &scene,
                    Parameters::default(),
                    format,
                ))
                .unwrap();
            assert_eq!(stats.route, Some(RenderRoute::GpuGraph));
            let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
            // Straight readback reconstructs RGB from quantized premultiplied
            // samples. The smallest alpha here allows up to four byte levels.
            assert_pixel(&pixels, 2, 3, expected, 4);
        }
    }
}

#[test]
fn turbulence_clamps_each_channel_before_compositing_its_sample() {
    let (mut renderer, mut surface) = renderer_and_surface(1, 1);
    let full = bounds(1, 1);
    let operation = Turbulence::try_new([0.71875, 0.6796875], 36.0)
        .unwrap()
        .with_octaves(9)
        .with_color_space(FilterColorSpace::Srgb);
    let mut scene = Scene::new();
    scene.fill(full, Color::BLACK);
    scene
        .turbulence_layer(
            Layer::new(),
            operation,
            TurbulenceRegion::try_new(full, full).unwrap(),
        )
        .unwrap();
    let pixels = render(&mut renderer, &mut surface, &scene);
    // The unbounded octave sum at (0.5, 0.5) is approximately
    // (0.45548, 0.41043, 1.09589, 0.42318). Over opaque black,
    // clamped blue contributes only the source alpha (108, versus 118).
    assert_pixel(&pixels, 0, 0, [49, 44, 108, 255], 2);
}

#[test]
fn stitching_snaps_each_axis_and_preserves_the_tile_under_filter_clipping() {
    let (mut renderer, mut surface) = renderer_and_surface(10, 6);
    let tile = bounds(10, 6);
    let operation = Turbulence::try_new([0.31, 0.29], 1.0)
        .unwrap()
        .with_octaves(3)
        .with_kind(TurbulenceKind::FractalNoise)
        .with_color_space(FilterColorSpace::Srgb);
    let stitched = operation.with_stitch(TurbulenceStitch::Stitch);
    let pixels = render(
        &mut renderer,
        &mut surface,
        &scene(stitched, Layer::new(), tile, tile),
    );

    // The least relative change snaps the 10-wide axis DOWN to 3/10 and
    // the 6-high axis UP to 2/6. The right and bottom centers interpolate
    // with wrapped lattice points, including on higher octaves.
    assert_pixel(&pixels, 0, 0, [155, 168, 176, 54], 4);
    assert_pixel(&pixels, 9, 0, [88, 86, 93, 150], 2);
    assert_pixel(&pixels, 0, 5, [153, 188, 158, 107], 2);
    assert_pixel(&pixels, 9, 5, [94, 114, 79, 178], 2);

    let snapped = Turbulence::try_new([3.0 / 10.0, 2.0 / 6.0], 1.0)
        .unwrap()
        .with_octaves(3)
        .with_kind(TurbulenceKind::FractalNoise)
        .with_stitch(TurbulenceStitch::Stitch)
        .with_color_space(FilterColorSpace::Srgb);
    let snapped_pixels = render(
        &mut renderer,
        &mut surface,
        &scene(snapped, Layer::new(), tile, tile),
    );
    assert_eq!(pixels.rgba(), snapped_pixels.rgba());

    let unstitched = render(
        &mut renderer,
        &mut surface,
        &scene(operation, Layer::new(), tile, tile),
    );
    assert_pixel(&unstitched, 9, 5, [123, 116, 133, 133], 2);

    let crop = Rect::new(1.0, 1.0, 8.0, 4.0);
    let cropped = render(
        &mut renderer,
        &mut surface,
        &scene(stitched, Layer::new(), crop, tile),
    );
    assert_pixel(&cropped, 8, 4, [128, 113, 100, 174], 2);
    assert_pixel(&cropped, 0, 0, [0; 4], 0);
    assert_pixel(&cropped, 9, 5, [0; 4], 0);
}

#[test]
fn stitching_wraps_both_vertices_of_a_fractionally_placed_one_cell_tile() {
    let (mut renderer, mut surface) = renderer_and_surface(6, 6);
    let tile = Rect::new(1.5, 0.0, 4.0, 6.0);
    let operation = Turbulence::try_new([0.24, 0.29], 1.0)
        .unwrap()
        .with_kind(TurbulenceKind::FractalNoise)
        .with_stitch(TurbulenceStitch::Stitch)
        .with_color_space(FilterColorSpace::Srgb);
    let pixels = render(
        &mut renderer,
        &mut surface,
        &scene(operation, Layer::new(), bounds(6, 6), tile),
    );
    // X frequency snaps to 1/4. Both lattice corners select the only X
    // vertex modulo the period, even at local x=5.25, where the raw corner
    // indices are 1 and 2. The first and last texels cover half a cell.
    assert_pixel(&pixels, 1, 2, [112, 110, 123, 54], 4); // x=1.75
    assert_pixel(&pixels, 4, 2, [115, 109, 128, 107], 3); // x=4.5
    assert_pixel(&pixels, 5, 2, [115, 109, 127, 54], 4); // x=5.25

    let crop = Rect::new(4.0, 2.0, 1.0, 1.0);
    let cropped = render(
        &mut renderer,
        &mut surface,
        &scene(operation, Layer::new(), crop, tile),
    );
    assert_pixel(&cropped, 4, 2, [115, 109, 128, 107], 3);
    assert_pixel(&cropped, 5, 2, [0; 4], 0);
}

#[test]
fn rotated_noise_uses_local_coordinates_and_the_transformed_result_clip() {
    let (mut renderer, mut surface) = renderer_and_surface(6, 6);
    let operation = Turbulence::try_new(FREQUENCY, 1.0)
        .unwrap()
        .with_kind(TurbulenceKind::FractalNoise)
        .with_color_space(FilterColorSpace::Srgb);
    let transform = Transform::try_new([0.0, 1.0, -1.0, 0.0, 5.0, 1.0]).unwrap();
    let scene = scene(
        operation,
        Layer::new().with_transform(transform),
        bounds(4, 4),
        Rect::new(1.0, 0.0, 3.0, 4.0),
    );
    let pixels = render(&mut renderer, &mut surface, &scene);
    // (x, y) -> (5 - y, 1 + x). The two observed centers come from
    // different local points in the same field.
    assert_pixel(&pixels, 1, 3, [144, 127, 160, 75], 3);
    assert_pixel(&pixels, 2, 4, [157, 155, 165, 54], 4);
    assert_pixel(&pixels, 1, 1, [0; 4], 0);
    assert_pixel(&pixels, 0, 3, [0; 4], 0);
}

#[test]
fn flat_fractal_noise_encodes_its_working_gray_and_applies_fractional_coverage_once() {
    let (mut renderer, mut surface) = renderer_and_surface(4, 2);
    let full = bounds(4, 2);
    let primitive = Rect::new(0.5, 0.0, 2.0, 2.0);
    // Both a zero field and an empty octave sum yield straight RGBA 0.5.
    // Linear 0.5 encodes as approximately 0.735 in sRGB. The outer halves
    // have coverage 0.5, changing alpha from 0.5 to 0.25.
    for (frequency, octaves, space, gray) in [
        ([0.0; 2], 1, FilterColorSpace::LinearSrgb, 188),
        (FREQUENCY, 0, FilterColorSpace::Srgb, 128),
    ] {
        let operation = Turbulence::try_new(frequency, 7.9)
            .unwrap()
            .with_octaves(octaves)
            .with_kind(TurbulenceKind::FractalNoise)
            .with_color_space(space);
        let pixels = render(
            &mut renderer,
            &mut surface,
            &scene(operation, Layer::new(), full, primitive),
        );
        for (x, alpha) in [(0, 64), (1, 128), (2, 64), (3, 0)] {
            let expected = if alpha == 0 {
                [0; 4]
            } else {
                [gray, gray, gray, alpha]
            };
            assert_pixel(&pixels, x, 0, expected, 3);
        }
        let usage = renderer.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::TurbulenceLatticeCandidates), 0);
        assert_eq!(usage.work(RenderWork::ScheduledTurbulenceOctaves), 0);
    }

    let default_kind = Turbulence::try_new([0.0; 2], 0.0).unwrap();
    let scene = scene(default_kind, Layer::new(), full, primitive);
    pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default())).unwrap();
    let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    assert!(pixels.rgba().iter().all(|channel| *channel == 0));
}

#[test]
fn fractional_copy_replaces_only_its_covered_footprint_even_with_transparent_noise() {
    let (mut renderer, mut surface) = renderer_and_surface(4, 1);
    let full = bounds(4, 1);
    let primitive = Rect::new(0.5, 0.0, 2.0, 1.0);
    let fractal = Turbulence::default()
        .with_kind(TurbulenceKind::FractalNoise)
        .with_color_space(FilterColorSpace::Srgb);
    // Copy lerps from the red backdrop to the premultiplied noise by
    // coverage. A half-covered gray edge is (.625, .125, .125, .75).
    // Transparent noise and zero layer opacity still replace covered pixels.
    for (operation, opacity, edge, center) in [
        (fractal, 1.0, [213, 43, 43, 191], [128; 4]),
        (Turbulence::default(), 1.0, [255, 0, 0, 128], [0; 4]),
        (fractal, 0.0, [255, 0, 0, 128], [0; 4]),
    ] {
        let mut scene = Scene::new();
        scene.fill(full, Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap());
        let layer = Layer::new()
            .with_composition(LayerComposition::normal_porter_duff(
                PorterDuffOperator::Copy,
            ))
            .try_opacity(opacity)
            .unwrap();
        scene
            .turbulence_layer(
                layer,
                operation,
                TurbulenceRegion::try_new(full, primitive).unwrap(),
            )
            .unwrap();
        let pixels = render(&mut renderer, &mut surface, &scene);
        assert_pixel(&pixels, 0, 0, edge, 2);
        assert_pixel(&pixels, 1, 0, center, 2);
        assert_pixel(&pixels, 2, 0, edge, 2);
        assert_pixel(&pixels, 3, 0, [255, 0, 0, 255], 0);
    }
}

#[test]
fn turbulence_charges_each_attempted_gradient_and_scheduled_pixel_octave() {
    let (mut renderer, mut surface) = renderer_and_surface(3, 2);
    let full = bounds(3, 2);
    let operation = Turbulence::try_new(FREQUENCY, 1.0).unwrap().with_octaves(3);
    render(
        &mut renderer,
        &mut surface,
        &scene(operation, Layer::new(), full, full),
    );
    let usage = renderer.last_attempt().unwrap().attempt().unwrap().usage();
    // The seed-1 Park–Miller stream tries 1,321 pairs to accept 1,024
    // disk gradients. The six scheduled pixels each evaluate three octaves.
    assert_eq!(usage.work(RenderWork::TurbulenceLatticeCandidates), 1321);
    assert_eq!(usage.work(RenderWork::ScheduledTurbulenceOctaves), 18);
}

#[test]
fn turbulence_work_refusal_keeps_the_prior_pixels_and_statistics() {
    let full = bounds(3, 2);
    for (quantity, limit) in [
        (RenderWork::TurbulenceLatticeCandidates, 1320),
        (RenderWork::ScheduledTurbulenceOctaves, 17),
    ] {
        let options = RenderAccountingOptions::new().with_limits(
            RenderLimits::new()
                .with_work_limit(quantity, limit)
                .unwrap(),
        );
        let mut renderer =
            pollster::block_on(Renderer::new(Options::default().with_accounting(options))).unwrap();
        let mut surface =
            pollster::block_on(renderer.create_headless(Size::new(3.0, 2.0), 1.0)).unwrap();
        let mut original = Scene::new();
        original.fill(full, Color::BLACK);
        let prior =
            pollster::block_on(renderer.render(&mut surface, &original, Parameters::default()))
                .unwrap();
        let operation = Turbulence::try_new(FREQUENCY, 1.0).unwrap().with_octaves(3);
        let scene = scene(operation, Layer::new(), full, full);
        let result =
            pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default()));
        assert!(result.unwrap_err().accounting_diagnostic().is_some());
        let report = renderer.last_attempt().unwrap();
        let attempt = report.attempt().unwrap();
        assert_eq!(report.status(), RenderAttemptStatus::Refused);
        assert_eq!(
            attempt.refusal().unwrap().quantity(),
            RenderQuantity::Work(quantity)
        );
        assert_eq!(attempt.usage().work(RenderWork::Submissions), 0);
        assert_eq!(renderer.stats(), prior);
        let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        for rgba in pixels.rgba().as_chunks::<4>().0 {
            assert_eq!(rgba, &[0, 0, 0, 255]);
        }
    }
}

#[test]
fn same_layer_backdrop_rejection_leaves_the_scene_intact() {
    let full = bounds(1, 1);
    let mut scene = Scene::new();
    scene.fill(full, Color::BLACK);
    let backdrop = BackdropFilterInput::try_new(
        FilterList::try_ops(vec![FilterOp::blur(FilterBlur::try_new(1.0).unwrap())]).unwrap(),
        BackdropCaptureBounds::try_new(full).unwrap(),
        None,
    )
    .unwrap();
    let result = scene.turbulence_layer(
        Layer::new().try_backdrop_filter(backdrop).unwrap(),
        Turbulence::try_new(FREQUENCY, 1.0).unwrap(),
        TurbulenceRegion::try_new(full, full).unwrap(),
    );
    assert!(result.is_err());
    assert_eq!(scene.len(), 1);
}

#[test]
fn offscreen_turbulence_copy_does_not_reach_lattice_setup() {
    use crate::scene::{LayerComposition, PorterDuffOperator};

    let options = RenderAccountingOptions::new().with_limits(
        RenderLimits::new()
            .with_work_limit(RenderWork::TurbulenceLatticeCandidates, 0)
            .unwrap(),
    );
    let mut renderer =
        pollster::block_on(Renderer::new(Options::default().with_accounting(options))).unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(1.0, 1.0), 1.0)).unwrap();
    let mut scene = Scene::new();
    scene.fill(bounds(1, 1), Color::BLACK);
    let offscreen = Rect::new(10.0, 0.0, 1.0, 1.0);
    scene
        .turbulence_layer(
            Layer::new().with_composition(LayerComposition::normal_porter_duff(
                PorterDuffOperator::Copy,
            )),
            Turbulence::try_new(FREQUENCY, 1.0).unwrap(),
            TurbulenceRegion::try_new(offscreen, offscreen).unwrap(),
        )
        .unwrap();
    let stats =
        pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default())).unwrap();
    assert_eq!(stats.route, Some(RenderRoute::DirectVello));
    let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    assert_pixel(&pixels, 0, 0, [0, 0, 0, 255], 0);
    let usage = renderer.last_attempt().unwrap().attempt().unwrap().usage();
    assert_eq!(usage.work(RenderWork::TurbulenceLatticeCandidates), 0);
    assert_eq!(usage.work(RenderWork::ScheduledTurbulenceOctaves), 0);
}
