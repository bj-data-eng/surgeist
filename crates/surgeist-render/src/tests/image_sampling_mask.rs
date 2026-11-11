use std::sync::Arc;

use crate::{
    Antialiasing, Capabilities, Color, Extend, Image, ImageBuffer, ImageRendering, Layer,
    PhysicalSize, Rect, ResolvedLayerAlphaMask, Scene, Size, Transform, frame::FrameContext,
    reference,
};

fn mask(size: PhysicalSize, alpha: &[u8], extend: Extend) -> Image {
    let bytes = alpha
        .iter()
        .flat_map(|alpha| [239, 17, 91, *alpha])
        .collect::<Vec<_>>();
    Image::from_rgba(
        Size::new(f64::from(size.width()), f64::from(size.height())),
        Arc::<[u8]>::from(bytes),
    )
    .unwrap()
    .rendering(ImageRendering::Pixelated)
    .extend(extend)
}

fn reference_alpha(image: Image, size: PhysicalSize, bounds: Rect) -> Vec<u8> {
    let source = ImageBuffer::try_new(
        size,
        [255; 4].repeat(usize::try_from(size.width() * size.height()).unwrap()),
    )
    .unwrap();
    reference::execute_transitional_resolved_mask_bridge_for_test(
        &source,
        Rect::new(0.0, 0.0, f64::from(size.width()), f64::from(size.height())),
        image,
        bounds,
    )
    .unwrap()
    .rgba()
    .as_chunks::<4>()
    .0
    .iter()
    .map(|pixel| pixel[3])
    .collect()
}

#[test]
fn pixelated_mask_fractional_integer_and_downscale_follow_virtual_lattice() {
    let image = mask(PhysicalSize::new(3, 1), &[0, 128, 255], Extend::Pad);
    // At 8/3 scale the nearest integer enlargement is 3. Virtual texel
    // coordinates 2.3125 and 5.6875 interpolate 5/16 and 11/16 at the steps.
    for (width, expected) in [
        (8, vec![0, 0, 40, 128, 128, 215, 255, 255]),
        (6, vec![0, 0, 128, 128, 255, 255]),
        (2, vec![32, 223]),
    ] {
        assert_eq!(
            reference_alpha(
                image.clone(),
                PhysicalSize::new(width, 1),
                Rect::new(0.0, 0.0, f64::from(width), 1.0)
            ),
            expected,
        );
    }
}

#[test]
fn pixelated_mask_anisotropic_lattice_interpolates_alpha_only() {
    let image = mask(PhysicalSize::new(2, 2), &[0, 64, 128, 255], Extend::Pad);
    assert_eq!(
        reference_alpha(
            image,
            PhysicalSize::new(5, 3),
            Rect::new(0.0, 0.0, 5.0, 3.0)
        ),
        [
            0, 0, 32, 64, 64, 64, 64, 112, 160, 160, 128, 128, 192, 255, 255
        ],
    );
}

#[test]
fn pixelated_mask_virtual_taps_preserve_extend_and_transparent_bounds() {
    for (extend, expected) in [
        (Extend::Pad, vec![0, 0, 0, 150, 255, 255, 255, 0]),
        (Extend::Reflect, vec![0, 0, 0, 150, 255, 255, 255, 0]),
        (Extend::Repeat, vec![15, 0, 0, 150, 255, 255, 195, 0]),
    ] {
        assert_eq!(
            reference_alpha(
                mask(PhysicalSize::new(2, 1), &[0, 255], extend),
                PhysicalSize::new(8, 1),
                Rect::new(0.0, 0.0, 6.8, 1.0),
            ),
            expected,
        );
    }
}

fn commands(image: Image, bounds: Rect, transform: Transform) -> crate::command::RenderCommands {
    let mut scene = Scene::new();
    scene.layer(
        Layer::new()
            .with_transform(transform)
            .with_resolved_alpha_mask(ResolvedLayerAlphaMask::try_new(image, bounds).unwrap()),
        |scene| {
            scene.fill(bounds, Color::BLACK);
        },
    );
    scene
        .normalize(
            Capabilities::CURRENT,
            &crate::accounting::Account::for_test(),
        )
        .unwrap()
}

#[test]
fn pixelated_mask_graph_parameters_use_transformed_basis_and_device_scale() {
    let image = mask(PhysicalSize::new(1, 1), &[128], Extend::Repeat);
    let context = FrameContext::try_new(
        Size::new(16.0, 16.0),
        1.5,
        Antialiasing::Msaa8,
        Color::TRANSPARENT,
    )
    .unwrap();
    let bytes = crate::pass::composite_parameter_bytes_for_test(
        commands(
            image.clone(),
            Rect::new(0.0, 0.0, 1.0, 1.0),
            Transform::try_new([3.0, 4.0, 0.0, 2.0, 1.0, 1.0]).unwrap(),
        ),
        context,
    )
    .unwrap();
    // Basis lengths 5 and 2 become device lengths 7.5 and 3 at DPR 1.5.
    let read = |offset| u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
    assert_eq!([read(56), read(60), read(88), read(92)], [8, 3, 3, 1]);
    let second = crate::pass::composite_parameter_bytes_for_test(
        commands(
            image,
            Rect::new(0.0, 0.0, 1.0, 1.0),
            Transform::scale(2.0, 3.0).unwrap(),
        ),
        context,
    )
    .unwrap();
    assert_eq!(u32::from_le_bytes(second[56..60].try_into().unwrap()), 3);
    assert_eq!(u32::from_le_bytes(second[60..64].try_into().unwrap()), 5);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn pixelated_mask_gpu_fractional_rotation_and_dpr_match_independent_alpha_oracle() {
    use crate::{Options, Parameters, RenderRoute, Renderer};
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    for (dpr, logical_size, transform, vertical) in [
        (
            1.0,
            Size::new(8.0, 1.0),
            Transform::scale(8.0 / 3.0, 1.0).unwrap(),
            false,
        ),
        (
            2.0,
            Size::new(4.0, 0.5),
            Transform::scale(4.0 / 3.0, 0.5).unwrap(),
            false,
        ),
        (
            1.0,
            Size::new(1.0, 8.0),
            Transform::try_new([0.0, 8.0 / 3.0, -1.0, 0.0, 1.0, 0.0]).unwrap(),
            true,
        ),
    ] {
        let mut scene = Scene::new();
        let bounds = Rect::new(0.0, 0.0, 3.0, 1.0);
        scene.layer(
            Layer::new()
                .with_transform(transform)
                .with_resolved_alpha_mask(
                    ResolvedLayerAlphaMask::try_new(
                        mask(PhysicalSize::new(3, 1), &[0, 128, 255], Extend::Pad),
                        bounds,
                    )
                    .unwrap(),
                ),
            |scene| {
                scene.fill(bounds, Color::try_rgba(1.0, 1.0, 1.0, 1.0).unwrap());
            },
        );
        let mut surface = pollster::block_on(renderer.create_headless(logical_size, dpr)).unwrap();
        let stats =
            pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default()))
                .unwrap();
        assert_eq!(stats.route, Some(RenderRoute::GpuGraph));
        let output = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        assert_eq!(
            output.size(),
            if vertical {
                PhysicalSize::new(1, 8)
            } else {
                PhysicalSize::new(8, 1)
            }
        );
        for (observed, expected) in output
            .rgba()
            .as_chunks::<4>()
            .0
            .iter()
            .map(|pixel| pixel[3])
            .zip([0_u8, 0, 40, 128, 128, 215, 255, 255])
        {
            assert!(
                observed.abs_diff(expected) <= 2,
                "alpha {observed} != {expected} (DPR {dpr}, vertical {vertical})"
            );
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn pixelated_mask_accounts_for_both_placements_and_preserves_publication_on_refusal() {
    use crate::{
        ImageQuality, Options, Parameters, RenderAccountingError, RenderAccountingOptions,
        RenderAttemptStatus, RenderLimits, RenderPhase, RenderQuantity, RenderRefusalReason,
        RenderRoute, RenderWork, Renderer,
    };

    let pixelated = mask(PhysicalSize::new(2, 1), &[0, 255], Extend::Pad);
    let ordinary = pixelated.clone().quality(ImageQuality::Medium);
    let size = Size::new(7.0, 3.0);
    let bounds = Rect::new(0.0, 0.0, 2.0, 1.0);
    let transform = Transform::scale(2.5, 1.0)
        .unwrap()
        .then(Transform::translation(1.0, 1.0).unwrap())
        .unwrap();
    let [ordinary, pixelated] = [ordinary, pixelated].map(|image| {
        let mut scene = Scene::new();
        scene.layer(
            Layer::new()
                .with_transform(transform)
                .with_resolved_alpha_mask(ResolvedLayerAlphaMask::try_new(image, bounds).unwrap()),
            |child| {
                child.fill(bounds, Color::BLACK);
            },
        );
        scene
    });

    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let mut surface = pollster::block_on(renderer.create_headless(size, 1.0)).unwrap();
    let coordinates = RenderWork::CoordinateVisits;
    let [ordinary_work, pixelated_work] = [&ordinary, &pixelated].map(|scene| {
        let stats = pollster::block_on(renderer.render(&mut surface, scene, Parameters::default()))
            .unwrap();
        assert_eq!(stats.route, Some(RenderRoute::GpuGraph));
        let report = renderer.last_attempt().unwrap();
        let attempt = report.attempt().unwrap();
        [
            RenderPhase::Attempt,
            RenderPhase::GpuPrepare,
            RenderPhase::Encode,
        ]
        .map(|phase| attempt.phase_usage(phase).work(coordinates))
    });
    assert_eq!(pixelated_work[1], ordinary_work[1] + 76);
    assert_eq!(pixelated_work[2], ordinary_work[2] + 76);
    let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    for (y, row) in [[0; 7], [0, 0, 0, 128, 255, 255, 0], [0; 7]]
        .into_iter()
        .enumerate()
    {
        for (x, alpha) in row.into_iter().enumerate() {
            let actual = super::support::pixel_rgba(&pixels, x as u32, y as u32)[3];
            assert!(
                actual.abs_diff(alpha) <= 2,
                "({x}, {y}): {actual} != {alpha}"
            );
        }
    }
    drop((renderer, surface));

    let cap = pixelated_work[0] - 1;
    assert!(ordinary_work[0] <= cap);
    let limits = RenderLimits::new()
        .with_work_limit(coordinates, cap)
        .unwrap();
    let options =
        Options::default().with_accounting(RenderAccountingOptions::new().with_limits(limits));
    let mut renderer = pollster::block_on(Renderer::new(options)).unwrap();
    let mut surface = pollster::block_on(renderer.create_headless(size, 1.0)).unwrap();
    let before_stats =
        pollster::block_on(renderer.render(&mut surface, &ordinary, Parameters::default()))
            .unwrap();
    let before_pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    let before_publication = surface.headless_publication_count_for_test();
    let error =
        pollster::block_on(renderer.render(&mut surface, &pixelated, Parameters::default()))
            .unwrap_err();
    let Some(RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic() else {
        panic!("expected coordinate refusal, got {error:?}");
    };
    assert_eq!(refusal.quantity(), RenderQuantity::Work(coordinates));
    assert_eq!(refusal.reason(), RenderRefusalReason::Limit { limit: cap });
    let report = renderer.last_attempt().unwrap();
    assert_eq!(report.status(), RenderAttemptStatus::Refused);
    assert_eq!(report.cleanup_fault(), None);
    assert_eq!(
        report
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::Submissions),
        0
    );
    assert_eq!(renderer.stats(), before_stats);
    assert_eq!(
        surface.headless_publication_count_for_test(),
        before_publication
    );
    assert_eq!(
        pollster::block_on(renderer.read_headless(&surface)).unwrap(),
        before_pixels
    );
    pollster::block_on(renderer.render(&mut surface, &ordinary, Parameters::default())).unwrap();
    assert_eq!(
        pollster::block_on(renderer.read_headless(&surface)).unwrap(),
        before_pixels
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn pixelated_mask_gpu_reuses_source_at_distinct_scales_and_preserves_extends() {
    use crate::{Options, Parameters, RenderRoute, Renderer};
    let shared = mask(PhysicalSize::new(3, 1), &[0, 128, 255], Extend::Pad);
    let mut scene = Scene::new();
    let cases = [
        (shared.clone(), 8.0, [0, 0, 40, 128, 128, 215, 255, 255]),
        (shared.clone(), 6.0, [0, 0, 128, 128, 255, 255, 0, 0]),
        (shared, 2.0, [32, 223, 0, 0, 0, 0, 0, 0]),
        (
            mask(PhysicalSize::new(2, 1), &[0, 255], Extend::Pad),
            6.8,
            [0, 0, 0, 150, 255, 255, 255, 0],
        ),
        (
            mask(PhysicalSize::new(2, 1), &[0, 255], Extend::Repeat),
            6.8,
            [15, 0, 0, 150, 255, 255, 195, 0],
        ),
        (
            mask(PhysicalSize::new(2, 1), &[0, 255], Extend::Reflect),
            6.8,
            [0, 0, 0, 150, 255, 255, 255, 0],
        ),
    ];
    let mut expected = Vec::new();
    for (row, (image, width, alpha)) in cases.into_iter().enumerate() {
        let y = f64::from(u32::try_from(row).unwrap());
        scene.layer(
            Layer::new().with_resolved_alpha_mask(
                ResolvedLayerAlphaMask::try_new(image, Rect::new(0.0, y, width, 1.0)).unwrap(),
            ),
            |scene| {
                scene.fill(
                    Rect::new(0.0, y, 8.0, 1.0),
                    Color::try_rgba(1.0, 1.0, 1.0, 1.0).unwrap(),
                );
            },
        );
        expected.extend(alpha);
    }
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(8.0, 6.0), 1.0)).unwrap();
    let stats =
        pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default())).unwrap();
    assert_eq!(stats.route, Some(RenderRoute::GpuGraph));
    let output = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    assert_eq!(output.size(), PhysicalSize::new(8, 6));
    for (index, (observed, expected)) in output
        .rgba()
        .as_chunks::<4>()
        .0
        .iter()
        .map(|pixel| pixel[3])
        .zip(expected)
        .enumerate()
    {
        assert!(
            observed.abs_diff(expected) <= 2,
            "alpha {observed} != {expected} at ({}, {})",
            index % 8,
            index / 8
        );
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn rendering_hints_reach_mask_gpu_sampling_without_collapsing_to_medium() {
    use crate::{Options, Parameters, Renderer};
    let mut scene = Scene::new();
    let mut expected = Vec::new();
    for (row, (hint, alpha)) in [
        (ImageRendering::Auto, [0, 26, 128, 230, 255]),
        (ImageRendering::Smooth, [0, 26, 128, 230, 255]),
        // Mitchell's padded step response at source coordinate 0.1 is
        // 131/1125, or alpha 29.6933; the opposite step is complementary.
        (ImageRendering::HighQuality, [0, 30, 128, 225, 255]),
        (ImageRendering::CrispEdges, [0, 0, 255, 255, 255]),
        (ImageRendering::Pixelated, [0, 0, 128, 255, 255]),
    ]
    .into_iter()
    .enumerate()
    {
        let y = f64::from(u32::try_from(row).unwrap());
        let bounds = Rect::new(0.0, y, 5.0, 1.0);
        scene.layer(
            Layer::new().with_resolved_alpha_mask(
                ResolvedLayerAlphaMask::try_new(
                    mask(PhysicalSize::new(2, 1), &[0, 255], Extend::Pad).rendering(hint),
                    bounds,
                )
                .unwrap(),
            ),
            |scene| {
                scene.fill(bounds, Color::try_rgba(1.0, 1.0, 1.0, 1.0).unwrap());
            },
        );
        expected.extend(alpha);
    }
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(5.0, 5.0), 1.0)).unwrap();
    pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default())).unwrap();
    let output = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    for (index, (observed, expected)) in output
        .rgba()
        .as_chunks::<4>()
        .0
        .iter()
        .map(|pixel| pixel[3])
        .zip(expected)
        .enumerate()
    {
        assert!(
            observed.abs_diff(expected) <= 2,
            "alpha {observed} != {expected} at ({}, {})",
            index % 5,
            index / 5
        );
    }
}
