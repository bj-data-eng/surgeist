//! Native feTile through resolved producer scenes and real GPU readback.

use super::convolution::{assert_pixel, bounds, renderer_and_surface};
use super::displacement::FORMATS;
use super::gaussian::{operation as gaussian, sigma};
use super::native_shadow::render;
use crate::{
    accounting::{RenderGenerationScope, RenderResource, RenderWork},
    effect::{FilterColorSpace, FilterDomains, FilterEdgeMode},
    gaussian::GaussianSource,
    geometry::{Rect, Transform},
    layer::{LayerComposition, PorterDuffOperator},
    paint::Color,
    scene::{Layer, Scene},
    stats::RenderRoute,
    surface::Parameters,
};

const CLEAR: [u8; 4] = [0; 4];
const RED: [u8; 4] = [255, 0, 0, 255];
const GREEN: [u8; 4] = [0, 255, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];
const YELLOW: [u8; 4] = [255, 255, 0, 255];

fn paint([red, green, blue, alpha]: [u8; 4]) -> Color {
    Color::try_rgba(
        f32::from(red) / 255.0,
        f32::from(green) / 255.0,
        f32::from(blue) / 255.0,
        f32::from(alpha) / 255.0,
    )
    .unwrap()
}

fn tile(producer: Scene, layer: Layer, source: Rect, filter: Rect, result: Rect) -> Scene {
    let mut scene = Scene::new();
    scene
        .tile_layer(
            layer,
            producer,
            FilterDomains::try_new(source, filter, result).unwrap(),
        )
        .unwrap();
    scene
}

#[test]
fn hard_filter_clips_one_producer_cell_without_shortening_its_period() {
    let (mut renderer, mut surface) = renderer_and_surface(12, 1);
    let full = bounds(12, 1);
    let source = Rect::new(-1.0, 0.0, 4.0, 1.0);
    let mut producer = Scene::new();
    for (x, color) in [(-1.0, YELLOW), (0.0, RED), (1.0, GREEN), (2.0, BLUE)] {
        producer.fill(Rect::new(x, 0.0, 1.0, 1.0), paint(color));
    }
    let scene = tile(producer, Layer::new(), source, full, full);
    for format in FORMATS {
        let stats = pollster::block_on(renderer.render_with_exact_graph_working_format_for_test(
            &mut surface,
            &scene,
            Parameters::default(),
            format,
        ))
        .unwrap();
        assert_eq!(stats.route, Some(RenderRoute::GpuGraph));
        let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        // The missing conceptual cell x=-1 repeats at x=3, 7, and 11.
        for (x, expected) in (0..12).zip([RED, GREEN, BLUE, CLEAR].into_iter().cycle()) {
            assert_pixel(&pixels, x, 0, expected, 1);
        }
        let attempt = renderer.last_attempt().unwrap();
        assert_eq!(
            attempt
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::ScheduledTileSamples),
            48
        );
    }
}

#[test]
fn negative_cycles_repeat_in_both_axes_inside_an_independent_result_clip() {
    let (mut renderer, mut surface) = renderer_and_surface(7, 6);
    let full = bounds(7, 6);
    let source = Rect::new(3.0, 2.0, 2.0, 2.0);
    let result = Rect::new(1.0, -1.0, 4.0, 5.0);
    let mut producer = Scene::new();
    for (x, y, color) in [
        (3.0, 2.0, RED),
        (4.0, 2.0, GREEN),
        (3.0, 3.0, BLUE),
        (4.0, 3.0, YELLOW),
    ] {
        producer.fill(Rect::new(x, y, 1.0, 1.0), paint(color));
    }
    let pixels = render(
        &mut renderer,
        &mut surface,
        &tile(producer, Layer::new(), source, full, result),
    );
    // The first reached pixel is at (1,0), a full period back in both axes.
    for (y, row) in (0..6).zip([
        [CLEAR, RED, GREEN, RED, GREEN, CLEAR, CLEAR],
        [CLEAR, BLUE, YELLOW, BLUE, YELLOW, CLEAR, CLEAR],
        [CLEAR, RED, GREEN, RED, GREEN, CLEAR, CLEAR],
        [CLEAR, BLUE, YELLOW, BLUE, YELLOW, CLEAR, CLEAR],
        [CLEAR; 7],
        [CLEAR; 7],
    ]) {
        for (x, expected) in (0..7).zip(row) {
            assert_pixel(&pixels, x, y, expected, 1);
        }
    }
}

#[test]
fn fractional_origin_interpolates_last_and_first_cells_across_the_seam() {
    let (mut renderer, mut surface) = renderer_and_surface(6, 1);
    let full = bounds(6, 1);
    let source = Rect::new(0.25, 0.0, 2.0, 1.0);
    let mut producer = Scene::new();
    producer.fill(Rect::new(0.25, 0.0, 1.0, 1.0), paint(RED));
    producer.fill(Rect::new(1.25, 0.0, 1.0, 1.0), paint(BLUE));
    let pixels = render(
        &mut renderer,
        &mut surface,
        &tile(producer, Layer::new(), source, full, full),
    );
    // The capture centers are 0.75 (red) and 1.75 (blue). At output x=0.5,
    // the left neighbor is the wrapped blue at -0.25 and contributes 1/4.
    // At x=1.5, red lies 3/4 away and contributes 1/4.
    let expected = [[191, 0, 64, 255], [64, 0, 191, 255]];
    for (x, pixel) in (0..6).zip(expected.into_iter().cycle()) {
        assert_pixel(&pixels, x, 0, pixel, 2);
    }
}

#[test]
fn a_fractionally_hard_clipped_source_cell_contributes_once_across_the_repeat() {
    let (mut renderer, mut surface) = renderer_and_surface(4, 1);
    let full = bounds(4, 1);
    let source = Rect::new(-0.5, 0.0, 2.0, 1.0);
    let mut producer = Scene::new();
    producer.fill(source, paint(RED));
    let pixels = render(
        &mut renderer,
        &mut surface,
        &tile(producer, Layer::new(), source, full, full),
    );
    // The conceptual cells have alpha 1/2 and 1 after hard clipping.
    // Every output center is midway between those cells, including when
    // the right neighbor wraps, yielding alpha 3/4 everywhere.
    for x in 0..4 {
        assert_pixel(&pixels, x, 0, [255, 0, 0, 191], 2);
    }
    let attempt = renderer.last_attempt().unwrap();
    assert_eq!(
        attempt
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::ScheduledTileSamples),
        64
    );
}

#[test]
fn rotated_and_sheared_fractional_tiles_keep_translucency_on_both_seams() {
    let (mut renderer, mut surface) = renderer_and_surface(18, 16);
    let filter = Rect::new(-4.0, -3.0, 15.0, 10.0);
    let source = Rect::new(0.375, 0.25, 2.5, 1.25);
    let result = Rect::new(-3.0, -2.0, 11.0, 7.0);
    for (affine, samples) in [
        ([0.8, 0.6, -0.6, 0.8, 8.7, 6.2], [(10, 8), (8, 9), (8, 8)]),
        ([1.2, 0.55, 0.4, 0.9, 6.2, 5.1], [(10, 8), (11, 9), (5, 5)]),
    ] {
        let mut producer = Scene::new();
        producer.fill(filter, Color::try_rgba(0.2, 0.6, 0.8, 0.625).unwrap());
        let pixels = render(
            &mut renderer,
            &mut surface,
            &tile(
                producer,
                Layer::new().with_transform(Transform::try_new(affine).unwrap()),
                source,
                filter,
                result,
            ),
        );
        // Inverse pixel centers for rotation are (2.82,.76), (1.82,2.76),
        // (1.22,1.96). For shear they are approximately (2.919,1.994),
        // (3.5,2.75), (-.919,1.006). Each set probes an X seam, a Y seam,
        // and an interior control, away from the final result boundary.
        for (x, y) in samples {
            assert_pixel(&pixels, x, y, [51, 153, 204, 159], 2);
        }
    }
}

#[test]
fn transparent_producers_still_execute_fractionally_bounded_copy() {
    let (mut renderer, mut surface) = renderer_and_surface(4, 1);
    let full = bounds(4, 1);
    let result = Rect::new(0.5, 0.0, 2.0, 1.0);
    let copy = LayerComposition::normal_porter_duff(PorterDuffOperator::Copy);
    for (source, has_ink) in [
        (Rect::new(0.0, 0.0, 0.0, 1.0), true),
        (Rect::new(6.0, 0.0, 2.0, 1.0), true),
        (full, false),
    ] {
        let mut producer = Scene::new();
        if has_ink {
            producer.fill(full, paint(BLUE));
        }
        let mut scene = Scene::new();
        scene.fill(full, paint(RED));
        scene
            .tile_layer(
                Layer::new().with_composition(copy),
                producer,
                FilterDomains::try_new(source, full, result).unwrap(),
            )
            .unwrap();
        let pixels = render(&mut renderer, &mut surface, &scene);
        // At the half cells, transparent Copy retains half the red backdrop.
        let half_red = [255, 0, 0, 128];
        for (x, expected) in (0..4).zip([half_red, CLEAR, half_red, RED]) {
            assert_pixel(&pixels, x, 0, expected, 1);
        }
    }
}

#[test]
fn empty_output_keeps_the_backdrop_and_schedules_no_image_pass() {
    let (mut renderer, mut surface) = renderer_and_surface(2, 1);
    let full = bounds(2, 1);
    let mut scene = Scene::new();
    scene.fill(full, paint(GREEN));
    let mut producer = Scene::new();
    producer.fill(Rect::new(-1.0e6, -1.0e6, 2.0e6, 2.0e6), paint(RED));
    scene
        .tile_layer(
            Layer::new().with_composition(LayerComposition::normal_porter_duff(
                PorterDuffOperator::Copy,
            )),
            producer,
            FilterDomains::try_new(full, full, Rect::new(2.0, 0.0, 1.0, 1.0)).unwrap(),
        )
        .unwrap();
    let stats =
        pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default())).unwrap();
    assert_eq!(stats.image_passes, 0);
    let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    assert_pixel(&pixels, 0, 0, GREEN, 0);
    assert_pixel(&pixels, 1, 0, GREEN, 0);
}

#[test]
fn a_huge_period_and_outside_paint_keep_a_two_pixel_capture_bounded() {
    let (mut renderer, mut surface) = renderer_and_surface(4, 1);
    let full = bounds(4, 1);
    let mut peak = [0; 3];
    for ((width, outside_paint), peak) in [(4.0, false), (1_048_576.0, false), (1_048_576.0, true)]
        .into_iter()
        .zip(&mut peak)
    {
        let mut producer = Scene::new();
        if outside_paint {
            producer.fill(Rect::new(-2.0e6, 0.0, 1_999_996.0, 1.0), paint(YELLOW));
        }
        producer.fill(Rect::new(0.0, 0.0, 1.0, 1.0), paint(RED));
        producer.fill(Rect::new(1.0, 0.0, 1.0, 1.0), paint(BLUE));
        let source = Rect::new(2.0 - width, 0.0, width, 1.0);
        let pixels = render(
            &mut renderer,
            &mut surface,
            &tile(producer, Layer::new(), source, full, full),
        );
        // The last two cells of either period contain ink. Output x=2 and 3
        // wrap far to its left, outside the two captured cells.
        for (x, expected) in (0..4).zip([RED, BLUE, CLEAR, CLEAR]) {
            assert_pixel(&pixels, x, 0, expected, 1);
        }
        *peak = renderer
            .last_attempt()
            .unwrap()
            .generation()
            .unwrap()
            .unwrap()
            .scope_usage(RenderGenerationScope::Effects)
            .peak(RenderResource::GpuBytes);
    }
    assert!(peak[0] > 0, "missing GPU allocation evidence");
    // The larger period would require at least 4 MiB as a dense texture.
    // Allow small fixed differences in renderer and resource-cache overhead.
    assert!(
        peak[1..].iter().all(|&bytes| bytes <= peak[0] + 65_536),
        "{peak:?}"
    );
}

#[test]
fn nested_gaussian_reads_the_halo_outside_the_tile_capture_and_restores_scope() {
    let (mut renderer, mut surface) = renderer_and_surface(9, 1);
    let full = bounds(8, 1);
    let source = Rect::new(2.0, 0.0, 2.0, 1.0);
    let gaussian_domain = bounds(5, 1);
    let mut impulse = Scene::new();
    impulse.fill(Rect::new(1.0, 0.0, 1.0, 1.0), paint([255; 4]));
    let mut producer = Scene::new();
    producer
        .gaussian_layer(
            Layer::new(),
            gaussian(sigma(), 0.0, FilterColorSpace::Srgb),
            GaussianSource::finite_with_edge(
                impulse,
                FilterDomains::try_new(
                    gaussian_domain,
                    gaussian_domain,
                    Rect::new(2.0, 0.0, 1.0, 1.0),
                )
                .unwrap(),
                FilterEdgeMode::None,
            ),
        )
        .unwrap();
    producer.fill(Rect::new(3.0, 0.0, 1.0, 1.0), paint(RED));
    let mut scene = tile(producer, Layer::new(), source, full, full);
    scene.fill(Rect::new(8.0, 0.0, 1.0, 1.0), paint(GREEN));
    let pixels = render(&mut renderer, &mut surface, &scene);
    // A one-texel offset under this Gaussian has normalized weight
    // 32768/139522. The input impulse is at x=1, outside the tile capture.
    let halo = [255, 255, 255, 60];
    for (x, expected) in (0..9).zip([halo, RED, halo, RED, halo, RED, halo, RED, GREEN]) {
        assert_pixel(&pixels, x, 0, expected, 2);
    }
}

#[test]
fn a_visible_producer_does_not_schedule_an_offscreen_tile_result() {
    let (mut renderer, mut surface) = renderer_and_surface(1, 1);
    let mut scene = Scene::new();
    scene.fill(bounds(1, 1), paint(GREEN));
    let mut producer = Scene::new();
    producer.fill(bounds(1, 1), paint(RED));
    scene
        .tile_layer(
            Layer::new(),
            producer,
            FilterDomains::try_new(bounds(1, 1), bounds(11, 1), Rect::new(10.0, 0.0, 1.0, 1.0))
                .unwrap(),
        )
        .unwrap();
    let stats =
        pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default())).unwrap();
    assert_eq!(stats.route, Some(RenderRoute::DirectVello));
    let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    assert_pixel(&pixels, 0, 0, GREEN, 0);
    let attempt = renderer.last_attempt().unwrap();
    assert_eq!(
        attempt
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::ScheduledTileSamples),
        0
    );
}

#[test]
fn a_partial_hard_clip_does_not_reuse_ink_outside_the_filter() {
    let (mut renderer, mut surface) = renderer_and_surface(4, 1);
    let full = bounds(4, 1);
    let source = Rect::new(-0.5, 0.0, 2.0, 1.0);
    for (painted, expected) in [
        (Rect::new(-0.5, 0.0, 0.5, 1.0), CLEAR),
        (Rect::new(0.0, 0.0, 1.5, 1.0), [255, 0, 0, 191]),
    ] {
        let mut producer = Scene::new();
        producer.fill(painted, paint(RED));
        let pixels = render(
            &mut renderer,
            &mut surface,
            &tile(producer, Layer::new(), source, full, full),
        );
        for x in 0..4 {
            assert_pixel(&pixels, x, 0, expected, 2);
        }
    }
}

#[test]
fn nested_morphology_qualifies_input_on_the_tile_capture_lattice() {
    use crate::morphology::{MorphologyInput, MorphologyOperator};

    let (mut renderer, mut surface) = renderer_and_surface(4, 1);
    let full = bounds(4, 1);
    let period = Rect::new(0.0, 0.0, 1.5, 1.0);
    let input = Rect::new(0.0, 0.0, 0.75, 0.75);
    let result = Rect::new(1.0, 0.0, 0.5, 0.75);
    let mut ink = Scene::new();
    ink.fill(input, paint([255; 4]));
    let producer = super::morphology::finite_scene(
        ink,
        FilterDomains::try_new(input, period, result).unwrap(),
        super::morphology::operation(
            MorphologyInput::new()
                .with_operator(MorphologyOperator::Dilate)
                .with_radii(0.9, 0.1),
            FilterColorSpace::Srgb,
        ),
        Layer::new(),
    );
    let pixels = render(
        &mut renderer,
        &mut surface,
        &tile(producer, Layer::new(), period, full, full),
    );
    // The capture samples a 1.5-unit period in two cells. A horizontal
    // radius of .9 spans one cell at 4/3 density and zero at the outer
    // unit density. The reached neighbor supplies visible white coverage.
    let actual = super::convolution::pixel(&pixels, 0, 0);
    assert_eq!(&actual[..3], &[255; 3], "{actual:?}");
    assert!(actual[3] > 0, "{actual:?}");
}
