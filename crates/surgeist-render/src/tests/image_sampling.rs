use std::sync::Arc;

use crate::{
    Color, Image, ImageFit, ImageQuality, ImageRendering, Layer, Options, Parameters, Rect,
    Renderer, Scene, Size, Transform, image::pixelated_multipliers,
};

use super::support::pixel_rgba;

fn black_white_image() -> Image {
    Image::from_rgba(
        Size::new(2.0, 1.0),
        Arc::<[u8]>::from([0, 0, 0, 255, 255, 255, 255, 255]),
    )
    .unwrap()
}

fn border_image_fixture() -> Image {
    Image::from_rgba(
        Size::new(4.0, 2.0),
        Arc::<[u8]>::from([
            255, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255, 255, 0, 255, 0, 255, 255, 255, 0, 255, 0,
            0, 255, 255, 255, 0, 255, 255, 0, 255, 255, 255,
        ]),
    )
    .unwrap()
}

#[test]
fn selecting_rendering_replaces_quality_without_changing_image_content() {
    let image = black_white_image();
    for (hint, quality) in [
        (ImageRendering::Auto, ImageQuality::Medium),
        (ImageRendering::Smooth, ImageQuality::Medium),
        (ImageRendering::HighQuality, ImageQuality::High),
        (ImageRendering::CrispEdges, ImageQuality::Low),
        (ImageRendering::Pixelated, ImageQuality::Medium),
    ] {
        let selected = image.clone().quality(ImageQuality::High).rendering(hint);
        assert_eq!(selected.id(), image.id());
        assert_eq!(selected.size(), image.size());
        assert_eq!(selected.rendering_hint(), Some(hint));
        assert_eq!(selected.sampling_quality(), quality);
        assert_ne!(selected, image);
        assert_eq!(selected.quality(ImageQuality::Medium), image);
    }
    assert_ne!(
        image.clone().rendering(ImageRendering::Pixelated),
        image.rendering(ImageRendering::Smooth),
    );
}

#[test]
fn pixelated_axes_use_independent_nearest_positive_integer_multiples() {
    let account = crate::accounting::Account::for_test();
    for (affine, expected) in [
        (kurbo::Affine::scale_non_uniform(2.5, 1.49), [3, 1]),
        (kurbo::Affine::scale_non_uniform(0.0, 0.25), [1, 1]),
        (
            kurbo::Affine::new([0.0, -3.0, 2.0, 0.0, 100.0, -100.0]),
            [3, 2],
        ),
        // The source-axis lengths are 5 and 2, despite their nonorthogonal basis.
        (kurbo::Affine::new([3.0, 4.0, 2.0, 0.0, 0.0, 0.0]), [5, 2]),
    ] {
        assert_eq!(pixelated_multipliers(affine, &account).unwrap(), expected);
    }
    for scale in [f64::NAN, f64::INFINITY, f64::from(u32::MAX) + 1.0] {
        assert!(pixelated_multipliers(kurbo::Affine::scale(scale), &account).is_err());
    }
}

fn render(renderer: &mut Renderer, scene: &Scene, width: u32, height: u32) -> crate::ImageBuffer {
    let mut surface = pollster::block_on(
        renderer.create_headless(Size::new(f64::from(width), f64::from(height)), 1.0),
    )
    .unwrap();
    pollster::block_on(renderer.render(&mut surface, scene, Parameters::default())).unwrap();
    pollster::block_on(renderer.read_headless(&surface)).unwrap()
}

fn assert_pixel(actual: [u8; 4], expected: [u8; 4]) {
    for (actual, expected) in actual.into_iter().zip(expected) {
        assert!(
            actual.abs_diff(expected) <= 2,
            "pixel channel {actual} differs from {expected}"
        );
    }
}

#[test]
fn image_region_isolates_texels_and_keeps_fractional_source_mapping() {
    let image = border_image_fixture().extend(crate::Extend::Repeat);
    let mut scene = Scene::new();
    scene.image_region(
        image.clone(),
        Rect::new(1.0, 1.0, 1.0, 1.0),
        Rect::new(0.0, 0.0, 6.0, 1.0),
    );
    scene.image_region(
        image.clone(),
        Rect::new(1.25, 0.0, 1.5, 1.0),
        Rect::new(0.0, 1.0, 6.0, 1.0),
    );
    scene.image_region(
        image,
        Rect::new(1.0, 1.0, 1.0, 1.0),
        Rect::new(0.0, 2.0, 6.0, 1.0),
    );
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let output = render(&mut renderer, &scene, 6, 3);
    assert_eq!(
        renderer.stats().route,
        Some(crate::RenderRoute::DirectVello)
    );
    for (x, value) in [0, 32, 96, 159, 223, 255].into_iter().enumerate() {
        assert_pixel(pixel_rgba(&output, x as u32, 0), [0, 0, 255, 255]);
        assert_pixel(pixel_rgba(&output, x as u32, 1), [value, value, value, 255]);
        assert_pixel(pixel_rgba(&output, x as u32, 2), [0, 0, 255, 255]);
    }
    let report = renderer.last_attempt().unwrap();
    let usage = report.attempt().unwrap().usage();
    assert_eq!(usage.work(crate::RenderWork::AtlasTexels), 3);
    assert_eq!(usage.work(crate::RenderWork::ImageSourceBytes), 64);
}

#[test]
fn image_region_keeps_pixelated_sampling_in_graph_capture() {
    assert_scaled_opacity_group_keeps_pixelated_lattice(|scene| {
        scene.image_region(
            border_image_fixture().rendering(ImageRendering::Pixelated),
            Rect::new(1.0, 0.0, 2.0, 1.0),
            Rect::new(0.0, 0.0, 2.0, 1.0),
        );
    });
}

#[test]
fn image_region_keeps_a_texel_crossed_by_a_rounded_endpoint() {
    let region = crate::image::ResolvedImageRegion::try_new(
        &border_image_fixture(),
        Rect::new(1.0_f64.next_up(), 0.0, 1.0, 1.0),
        &crate::accounting::Account::for_test(),
    )
    .unwrap();
    assert_eq!(region.window().first_pixel(), 1);
    assert_eq!(region.window().size(), crate::PhysicalSize::new(2, 1));
    assert_eq!(region.local().x(), f64::EPSILON);
}

#[test]
fn image_region_rejects_invalid_source_and_keeps_the_published_frame() {
    let image = border_image_fixture();
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(3.0, 1.0), 1.0)).unwrap();
    let target = Rect::new(0.0, 0.0, 3.0, 1.0);
    let mut good = Scene::new();
    good.image_region(image.clone(), Rect::new(2.0, 0.0, 1.0, 1.0), target);
    pollster::block_on(renderer.render(&mut surface, &good, Parameters::default())).unwrap();
    let stats = renderer.stats();
    for source in [
        Rect::new(-0.25, 0.0, 1.0, 1.0),
        Rect::new(3.5, 0.0, 1.0, 1.0),
        Rect::new(2.0, 0.0, 0.0, 1.0),
        Rect::new(f64::from_bits(1), 0.0, 4.0, 1.0),
    ] {
        let mut invalid = Scene::new();
        invalid.image_region(image.clone(), source, target);
        let error =
            pollster::block_on(renderer.render(&mut surface, &invalid, Parameters::default()))
                .unwrap_err();
        assert_eq!(error.code(), crate::ErrorCode::InvalidInput);
        assert_eq!(
            error.invalid_value_diagnostic().unwrap().field(),
            "image source region x"
        );
        assert_eq!(renderer.stats(), stats);
        let output = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        for x in 0..3 {
            assert_pixel(pixel_rgba(&output, x, 0), [255; 4]);
        }
    }
    pollster::block_on(renderer.render(&mut surface, &good, Parameters::default())).unwrap();
}

#[test]
fn image_region_atlas_refusal_keeps_the_prefix_and_allows_retry() {
    use crate::{
        RenderAccountingError, RenderAccountingOptions, RenderAttemptStatus, RenderLimits,
        RenderQuantity, RenderRefusalReason, RenderResource, RenderWork,
    };

    let image = border_image_fixture();
    let limits = RenderLimits::new()
        .with_work_limit(RenderWork::ImageSourceBytes, 32)
        .unwrap();
    let options =
        Options::default().with_accounting(RenderAccountingOptions::new().with_limits(limits));
    let mut renderer = pollster::block_on(Renderer::new(options)).unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(4.0, 1.0), 1.0)).unwrap();
    let mut scene = Scene::new();
    scene.image_region(
        image.clone(),
        Rect::new(1.0, 1.0, 1.0, 1.0),
        Rect::new(0.0, 0.0, 4.0, 1.0),
    );
    pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default())).unwrap();
    let stats = renderer.stats();
    let mut excessive = scene.clone();
    excessive.image_region(
        image,
        Rect::new(3.0, 1.0, 1.0, 1.0),
        Rect::new(2.0, 0.0, 2.0, 1.0),
    );
    let error =
        pollster::block_on(renderer.render(&mut surface, &excessive, Parameters::default()))
            .unwrap_err();
    let Some(RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic() else {
        panic!("expected typed atlas byte refusal: {error}");
    };
    assert_eq!(
        refusal.quantity(),
        RenderQuantity::Work(RenderWork::ImageSourceBytes)
    );
    assert_eq!(refusal.reason(), RenderRefusalReason::Limit { limit: 32 });
    assert_eq!(refusal.current(), 32);
    assert_eq!(refusal.amount(), 32);
    let report = renderer.last_attempt().unwrap();
    assert_eq!(report.status(), RenderAttemptStatus::Refused);
    assert_eq!(report.cleanup_fault(), None);
    let usage = report.attempt().unwrap().usage();
    assert_eq!(usage.work(RenderWork::ImageSourceBytes), 32);
    assert_eq!(usage.work(RenderWork::AtlasTexels), 1);
    assert_eq!(usage.live(RenderResource::LocalAtlasSourceByteQuota), 0);
    assert_eq!(usage.live(RenderResource::LocalAtlasTexelQuota), 0);
    assert_eq!(renderer.stats(), stats);

    let unchanged = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default())).unwrap();
    let retried = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    assert_eq!(retried.rgba(), unchanged.rgba());
    for x in 0..4 {
        assert_pixel(pixel_rgba(&retried, x, 0), [0, 0, 255, 255]);
    }
}

#[test]
fn fractional_pixelated_scaling_keeps_integer_blocks_and_smooths_the_remainder() {
    // At 2.5x, nearest expands 2 pixels to 6 before smooth scaling to 5.
    // The middle destination center is halfway between black and white.
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    for (hint, expected) in [
        (ImageRendering::Auto, [0, 26, 128, 230, 255]),
        (ImageRendering::Smooth, [0, 26, 128, 230, 255]),
        (ImageRendering::CrispEdges, [0, 0, 255, 255, 255]),
        (ImageRendering::Pixelated, [0, 0, 128, 255, 255]),
    ] {
        let mut scene = Scene::new();
        scene.image(
            black_white_image().rendering(hint),
            Rect::new(0.0, 0.0, 5.0, 1.0),
            ImageFit::Fill,
        );
        let output = render(&mut renderer, &scene, 5, 1);
        for (x, value) in expected.into_iter().enumerate() {
            assert_pixel(pixel_rgba(&output, x as u32, 0), [value, value, value, 255]);
        }
    }
}

#[test]
fn integer_pixelated_scaling_and_downscaling_follow_the_two_stage_rule() {
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    for (width, expected) in [(6, vec![0, 0, 0, 255, 255, 255]), (1, vec![128])] {
        let mut scene = Scene::new();
        scene.image(
            black_white_image().rendering(ImageRendering::Pixelated),
            Rect::new(0.0, 0.0, f64::from(width), 1.0),
            ImageFit::Fill,
        );
        let output = render(&mut renderer, &scene, width, 1);
        for (x, value) in expected.into_iter().enumerate() {
            assert_pixel(pixel_rgba(&output, x as u32, 0), [value, value, value, 255]);
        }
    }
}

#[test]
fn anisotropic_pixelated_scaling_uses_a_distinct_lattice_for_each_axis() {
    let image = Image::from_rgba(
        Size::new(2.0, 2.0),
        Arc::<[u8]>::from([
            0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 255,
        ]),
    )
    .unwrap()
    .rendering(ImageRendering::Pixelated);
    let mut scene = Scene::new();
    scene.image(image, Rect::new(0.0, 0.0, 5.0, 3.0), ImageFit::Fill);
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let output = render(&mut renderer, &scene, 5, 3);
    let expected = [[0, 0, 128, 255, 255], [128; 5], [255, 255, 128, 0, 0]];
    for (y, row) in expected.into_iter().enumerate() {
        for (x, value) in row.into_iter().enumerate() {
            assert_pixel(
                pixel_rgba(&output, x as u32, y as u32),
                [value, value, value, 255],
            );
        }
    }
}

#[test]
fn pixelated_smoothing_uses_premultiplied_colors_at_transparent_edges() {
    let image = Image::from_rgba(
        Size::new(2.0, 1.0),
        Arc::<[u8]>::from([255, 0, 0, 255, 0, 0, 255, 0]),
    )
    .unwrap()
    .rendering(ImageRendering::Pixelated);
    let mut scene = Scene::new();
    scene.image(image, Rect::new(0.0, 0.0, 5.0, 1.0), ImageFit::Fill);
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let output = render(&mut renderer, &scene, 5, 1);
    assert_pixel(pixel_rgba(&output, 2, 0), [255, 0, 0, 128]);
}

#[test]
fn one_image_can_use_distinct_pixelated_scales_in_a_single_scene() {
    let image = black_white_image().rendering(ImageRendering::Pixelated);
    let mut scene = Scene::new();
    scene.image(image.clone(), Rect::new(0.0, 0.0, 5.0, 1.0), ImageFit::Fill);
    scene.fill(
        Rect::new(0.0, 1.0, 5.0, 1.0),
        Color::try_rgba(1.0, 1.0, 1.0, 1.0).unwrap(),
    );
    scene.image(image, Rect::new(0.0, 2.0, 4.0, 1.0), ImageFit::Fill);
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let output = render(&mut renderer, &scene, 5, 3);
    for (x, value) in [0, 0, 128, 255, 255].into_iter().enumerate() {
        assert_pixel(pixel_rgba(&output, x as u32, 0), [value, value, value, 255]);
    }
    for (x, value) in [0, 0, 255, 255].into_iter().enumerate() {
        assert_pixel(pixel_rgba(&output, x as u32, 2), [value, value, value, 255]);
    }
}

#[test]
fn pixelated_image_paint_keeps_shape_geometry_while_scaling_its_brush() {
    let mut scene = Scene::new();
    scene.layer(
        Layer::new().with_transform(Transform::scale(2.5, 1.0).unwrap()),
        |scene| {
            scene.fill(
                Rect::new(0.0, 0.0, 2.0, 1.0),
                crate::Paint::image(black_white_image().rendering(ImageRendering::Pixelated)),
            );
        },
    );
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let output = render(&mut renderer, &scene, 5, 1);
    for (x, value) in [0, 0, 128, 255, 255].into_iter().enumerate() {
        assert_pixel(pixel_rgba(&output, x as u32, 0), [value, value, value, 255]);
    }
}

#[test]
fn pixelated_image_in_a_scaled_opacity_group_keeps_its_final_sampling_lattice() {
    assert_scaled_opacity_group_keeps_pixelated_lattice(|scene| {
        scene.fill(
            Rect::new(0.0, 0.0, 2.0, 1.0),
            crate::Paint::image(black_white_image().rendering(ImageRendering::Pixelated)),
        );
    });
}

#[test]
fn fitted_pixelated_image_command_in_a_scaled_opacity_group_keeps_its_sampling_lattice() {
    assert_scaled_opacity_group_keeps_pixelated_lattice(|scene| {
        scene.image(
            black_white_image().rendering(ImageRendering::Pixelated),
            Rect::new(0.0, 0.0, 2.0, 1.0),
            ImageFit::Fill,
        );
    });
}

fn assert_scaled_opacity_group_keeps_pixelated_lattice(draw: impl FnOnce(&mut Scene)) {
    let mut scene = Scene::new();
    scene.layer(
        Layer::new()
            .with_transform(Transform::scale(2.5, 1.0).unwrap())
            .try_opacity(0.5)
            .unwrap()
            .with_resolved_alpha_mask(
                crate::ResolvedLayerAlphaMask::try_new(
                    Image::from_rgba(Size::new(1.0, 1.0), Arc::<[u8]>::from([255; 4])).unwrap(),
                    Rect::new(0.0, 0.0, 2.0, 1.0),
                )
                .unwrap(),
            ),
        draw,
    );
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let output = render(&mut renderer, &scene, 5, 1);
    assert_eq!(renderer.stats().route, Some(crate::RenderRoute::GpuGraph));
    for (x, value) in [0, 0, 128, 255, 255].into_iter().enumerate() {
        assert_pixel(pixel_rgba(&output, x as u32, 0), [value, value, value, 128]);
    }
}

#[test]
fn pixelated_extent_refusal_preserves_published_pixels_and_statistics() {
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(5.0, 1.0), 1.0)).unwrap();
    let mut accepted = Scene::new();
    accepted.image(
        black_white_image().rendering(ImageRendering::Pixelated),
        Rect::new(0.0, 0.0, 5.0, 1.0),
        ImageFit::Fill,
    );
    pollster::block_on(renderer.render(&mut surface, &accepted, Parameters::default())).unwrap();
    let before = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    let stats = renderer.stats();
    let mut oversized = Scene::new();
    oversized.image(
        black_white_image().rendering(ImageRendering::Pixelated),
        Rect::new(0.0, 0.0, 131_072.0, 1.0),
        ImageFit::Fill,
    );
    assert!(
        pollster::block_on(renderer.render(&mut surface, &oversized, Parameters::default()))
            .is_err()
    );
    assert_eq!(renderer.stats(), stats);
    let after = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    assert_eq!(before.rgba(), after.rgba());
    // The same surface and renderer remain usable after the refusal.
    pollster::block_on(renderer.render(&mut surface, &accepted, Parameters::default())).unwrap();
    let retried = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    assert_eq!(before.rgba(), retried.rgba());
}

#[test]
fn pixelated_graph_keeps_an_integer_source_axis_crisp_beside_a_fractional_axis() {
    let image = Image::from_rgba(
        Size::new(2.0, 3.0),
        Arc::<[u8]>::from([
            0, 0, 0, 255, 0, 0, 0, 255, 128, 128, 128, 255, 128, 128, 128, 255, 255, 255, 255, 255,
            255, 255, 255, 255,
        ]),
    )
    .unwrap()
    .rendering(ImageRendering::Pixelated);
    let bounds = Rect::new(0.0, 0.0, 2.0, 3.0);
    let mask = crate::ResolvedLayerAlphaMask::try_new(
        Image::from_rgba(Size::new(1.0, 1.0), Arc::<[u8]>::from([255; 4])).unwrap(),
        bounds,
    )
    .unwrap();
    let mut scene = Scene::new();
    scene.layer(
        Layer::new()
            .with_transform(Transform::scale(2.5, 2.0).unwrap())
            .with_resolved_alpha_mask(mask),
        |scene| {
            scene.fill(bounds, crate::Paint::image(image));
        },
    );
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let output = render(&mut renderer, &scene, 5, 6);
    assert_eq!(renderer.stats().route, Some(crate::RenderRoute::GpuGraph));
    for (y, value) in [0, 0, 128, 128, 255, 255].into_iter().enumerate() {
        for x in 0..5 {
            assert_pixel(pixel_rgba(&output, x, y as u32), [value, value, value, 255]);
        }
    }
}

#[test]
fn high_quality_image_sampling_smooths_without_losing_opaque_coverage() {
    let mut scene = Scene::new();
    scene.image(
        black_white_image().rendering(ImageRendering::HighQuality),
        Rect::new(0.0, 0.0, 5.0, 1.0),
        ImageFit::Fill,
    );
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let output = render(&mut renderer, &scene, 5, 1);
    assert_pixel(pixel_rgba(&output, 2, 0), [128, 128, 128, 255]);
    assert!(pixel_rgba(&output, 1, 0)[0] > 0);
    assert!(pixel_rgba(&output, 3, 0)[0] < 255);
    for x in 0..5 {
        assert_eq!(pixel_rgba(&output, x, 0)[3], 255);
    }
}
