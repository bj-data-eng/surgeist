//! Independent public offset oracles. Readback is straight RGBA8; sampling
//! expectations below are derived from covered premultiplied source cells.
use super::convolution::{assert_pixel, bounds, image, pixel, renderer_and_surface};
use crate::{
    Color, FilterDomains, Image, ImageBuffer, ImageFit, ImageRendering, Layer, OffsetInput,
    Parameters, PreparedOffset, Rect, RenderAccountingContext, Renderer, Scene, Surface, Transform,
    prepare_offset,
};

pub(super) fn operation(
    input: OffsetInput,
    source: Rect,
    filter: Rect,
    result: Rect,
) -> PreparedOffset {
    prepare_offset(
        input,
        FilterDomains::try_new(source, filter, result).unwrap(),
        &mut RenderAccountingContext::default(),
    )
    .unwrap()
}

pub(super) fn image_scene(source: Image, destination: Rect, effect: PreparedOffset) -> Scene {
    let mut scene = Scene::new();
    scene.layer(Layer::new().with_offset(effect), |child| {
        child.image(source, destination, ImageFit::Stretch);
    });
    scene
}

pub(super) fn render(renderer: &mut Renderer, surface: &mut Surface, scene: &Scene) -> ImageBuffer {
    let stats = pollster::block_on(renderer.render(surface, scene, Parameters::default())).unwrap();
    assert_eq!(renderer.stats(), stats);
    pollster::block_on(renderer.read_headless(surface)).unwrap()
}

const HINTS: [ImageRendering; 5] = [
    ImageRendering::Auto,
    ImageRendering::Smooth,
    ImageRendering::HighQuality,
    ImageRendering::CrispEdges,
    ImageRendering::Pixelated,
];

#[test]
fn public_offset_default_and_signed_integer_translation_preserve_every_hint() {
    let (mut renderer, mut surface) = renderer_and_surface(5, 3);
    let full = bounds(5, 3);
    let source = Rect::new(1.0, 1.0, 3.0, 1.0);
    let colors = [[255, 0, 0, 255], [0, 255, 0, 255], [0, 0, 255, 255]];
    let input = image(3, 1, colors.concat());
    for hint in HINTS {
        for (dx, dy) in [(0.0, 0.0), (1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)] {
            let effect = operation(
                OffsetInput::new()
                    .with_dx(dx)
                    .with_dy(dy)
                    .with_rendering(hint),
                source,
                full,
                full,
            );
            let output = render(
                &mut renderer,
                &mut surface,
                &image_scene(input.clone(), source, effect),
            );
            assert_eq!(renderer.stats().route, Some(crate::RenderRoute::GpuGraph));
            for y in 0..3 {
                for x in 0..5 {
                    let index = i64::from(x) - 1 - dx as i64;
                    let expected = if i64::from(y) == 1 + dy as i64 && (0..3).contains(&index) {
                        colors[index as usize]
                    } else {
                        [0; 4]
                    };
                    assert_pixel(&output, x, y, expected, 1);
                }
            }
        }
    }
}

#[test]
fn public_offset_fractional_logical_displacement_copies_an_aligned_device_lattice() {
    use crate::{Options, Size};
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(2.0, 0.5), 2.0)).unwrap();
    let source = Rect::new(0.0, 0.0, 1.0, 0.5);
    let full = Rect::new(0.0, 0.0, 2.0, 0.5);
    for hint in HINTS {
        let effect = operation(
            OffsetInput::new().with_dx(0.5).with_rendering(hint),
            source,
            full,
            full,
        );
        let output = render(
            &mut renderer,
            &mut surface,
            &image_scene(
                image(2, 1, vec![255, 0, 0, 255, 0, 0, 255, 255]),
                source,
                effect,
            ),
        );
        // Logical dx.5 at rho2 is exactly one actual capture texel. HighQuality
        // must copy, rather than smooth its integer phase with Mitchell weights.
        for (x, expected) in [[0; 4], [255, 0, 0, 255], [0, 0, 255, 255], [0; 4]]
            .into_iter()
            .enumerate()
        {
            assert_pixel(&output, x as u32, 0, expected, 1);
        }
    }
}

#[test]
fn public_offset_fractional_sampling_keeps_input_and_output_coverage_independent() {
    let (mut renderer, mut surface) = renderer_and_surface(4, 3);
    let full = bounds(4, 3);
    let destination = Rect::new(1.0, 1.0, 1.0, 1.0);
    for hint in [
        ImageRendering::Auto,
        ImageRendering::Smooth,
        ImageRendering::Pixelated,
    ] {
        for (source, filter, result, expected) in [
            (destination, full, full, [128, 128]),
            (Rect::new(1.0, 1.0, 0.5, 1.0), full, full, [64, 64]),
            (
                Rect::new(1.0, 1.0, 0.5, 1.0),
                full,
                Rect::new(1.5, 0.0, 2.5, 3.0),
                [32, 64],
            ),
            (destination, Rect::new(1.5, 0.0, 2.5, 3.0), full, [32, 64]),
        ] {
            let effect = operation(
                OffsetInput::new().with_dx(0.5).with_rendering(hint),
                source,
                filter,
                result,
            );
            let output = render(
                &mut renderer,
                &mut surface,
                &image_scene(image(1, 1, vec![255; 4]), destination, effect),
            );
            for (x, alpha) in [(1, expected[0]), (2, expected[1])] {
                assert_pixel(&output, x, 1, [255, 255, 255, alpha], 2);
            }
            assert_pixel(&output, 0, 1, [0; 4], 1);
            assert_pixel(&output, 3, 1, [0; 4], 1);
        }
    }
    // Mirrored x/y cases independently exercise both inverse-map axes. A
    // half-texel translation on both axes splits one cell over four quarters.
    for (dx, dy, cells) in [
        (-0.5, 0.0, [(0, 1, 128), (1, 1, 128), (2, 1, 0), (3, 1, 0)]),
        (0.0, 0.5, [(1, 1, 128), (1, 2, 128), (1, 0, 0), (0, 0, 0)]),
        (0.0, -0.5, [(1, 0, 128), (1, 1, 128), (1, 2, 0), (0, 0, 0)]),
        (0.5, 0.5, [(1, 1, 64), (2, 1, 64), (1, 2, 64), (2, 2, 64)]),
    ] {
        let effect = operation(
            OffsetInput::new().with_dx(dx).with_dy(dy),
            destination,
            full,
            full,
        );
        let output = render(
            &mut renderer,
            &mut surface,
            &image_scene(image(1, 1, vec![255; 4]), destination, effect),
        );
        for (x, y, alpha) in cells {
            assert_pixel(
                &output,
                x,
                y,
                if alpha == 0 {
                    [0; 4]
                } else {
                    [255, 255, 255, alpha]
                },
                2,
            );
        }
    }
}

#[test]
fn public_offset_nearest_ties_and_fractional_mitchell_have_distinct_oracles() {
    let (mut renderer, mut surface) = renderer_and_surface(9, 5);
    let full = bounds(9, 5);
    let pixels = (0..5)
        .flat_map(|_| (0..9).flat_map(|x| if x == 4 { [255; 4] } else { [0; 4] }))
        .collect();
    let source = image(9, 5, pixels);
    for (hint, expected) in [
        (ImageRendering::CrispEdges, [0, 255, 0, 0]),
        (ImageRendering::Auto, [0, 128, 128, 0]),
        (ImageRendering::HighQuality, [0, 136, 136, 0]),
    ] {
        let effect = operation(
            OffsetInput::new().with_dx(0.5).with_rendering(hint),
            full,
            full,
            full,
        );
        let output = render(
            &mut renderer,
            &mut surface,
            &image_scene(source.clone(), full, effect),
        );
        // Mitchell w(.5)=77/144; w(1.5)=-5/144 clamps to0. A constant
        // vertical stripe sums the other axis's cubic weights to1.
        for (x, alpha) in (3..7).zip(expected) {
            assert_pixel(
                &output,
                x,
                2,
                if alpha == 0 {
                    [0; 4]
                } else {
                    [255, 255, 255, alpha]
                },
                2,
            );
        }
    }
    // Two adjacent opaque cells produce 154/144 at their midpoint; the
    // declared premultiplied clamp restores alpha and RGB to1.
    let source = image(
        9,
        5,
        (0..5)
            .flat_map(|_| (0..9).flat_map(|x| if x == 4 || x == 5 { [255; 4] } else { [0; 4] }))
            .collect(),
    );
    let effect = operation(
        OffsetInput::new()
            .with_dx(0.5)
            .with_rendering(ImageRendering::HighQuality),
        full,
        full,
        full,
    );
    let output = render(
        &mut renderer,
        &mut surface,
        &image_scene(source, full, effect),
    );
    assert_pixel(&output, 5, 2, [255; 4], 1);
}

#[test]
fn public_offset_interpolates_captured_premultiplied_numeric_color() {
    let (mut renderer, mut surface) = renderer_and_surface(4, 3);
    let full = bounds(4, 3);
    for (row, expected) in [
        (
            [[0, 0, 0, 255], [0, 0, 0, 255], [255; 4], [255; 4]],
            [128, 128, 128, 255],
        ),
        (
            [
                [255, 0, 0, 128],
                [255, 0, 0, 128],
                [0, 0, 255, 255],
                [0, 0, 255, 255],
            ],
            [85, 0, 170, 192],
        ),
    ] {
        let source = image(4, 3, [row.concat(), row.concat(), row.concat()].concat());
        let effect = operation(OffsetInput::new().with_dx(0.5), full, full, full);
        let scene = image_scene(source, full, effect);
        let output = render(&mut renderer, &mut surface, &scene);
        // Numeric midpoint is .5 (not linear-light encoded .735). The second
        // case is premul(.25,0,.5,.75), hence straight RGB(1/3,0,2/3).
        assert_pixel(&output, 2, 1, expected, 2);
        pollster::block_on(renderer.render_with_exact_graph_working_format_for_test(
            &mut surface,
            &scene,
            Parameters::default(),
            crate::resource::WorkingFormat::ReducedPrecision,
        ))
        .unwrap();
        let output = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        assert_pixel(&output, 2, 1, expected, 2);
    }
}

#[test]
fn public_offset_moves_nested_vectors_inside_the_complete_outer_affine() {
    let (mut renderer, mut surface) = renderer_and_surface(18, 14);
    let source = Rect::new(1.0, 1.0, 3.0, 3.0);
    let full = bounds(7, 6);
    for (affine, dx, dy, inside, outside) in [
        ([-1.0, 0.0, 0.0, 1.0, 8.0, 0.0], 1.0, 0.0, (3, 2), (6, 2)),
        ([0.0, 1.0, -1.0, 0.0, 8.0, 0.0], 1.0, 0.0, (5, 4), (5, 1)),
        ([2.0, 0.5, 1.0, 2.0, 0.0, 0.0], 1.5, 0.25, (12, 8), (4, 3)),
    ] {
        let effect = operation(
            OffsetInput::new().with_dx(dx).with_dy(dy),
            source,
            full,
            full,
        );
        let mut scene = Scene::new();
        scene.layer(
            Layer::new()
                .with_offset(effect)
                .with_transform(Transform::try_new(affine).unwrap()),
            |child| {
                child.layer(Layer::new().try_opacity(0.5).unwrap(), |nested| {
                    nested.fill(source, Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap());
                });
            },
        );
        pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default()))
            .unwrap_or_else(|error| panic!("affine{affine:?}, dx{dx},dy{dy}: {error:?}"));
        let output = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        assert_pixel(&output, inside.0, inside.1, [255, 0, 0, 128], 2);
        assert_pixel(&output, outside.0, outside.1, [0; 4], 2);
    }
}

#[test]
fn public_offset_fractional_image_plane_follows_full_affine_inverse() {
    use crate::ImageQuality;
    let (mut renderer, mut surface) = renderer_and_surface(24, 24);
    let domain = bounds(8, 8);
    let pixels = (0..8)
        .flat_map(|y| {
            (0..8).flat_map(move |x| {
                let red = (((f64::from(x) + 0.5 + 2.0 * (f64::from(y) + 0.5)) / 20.0) * 255.0)
                    .round() as u8;
                [red, 0, 0, 255]
            })
        })
        .collect();
    // Medium reconstruction retains the image's explicitly authored texel-center
    // plane; this oracle does not substitute a vector producer's raster phase.
    let source = image(8, 8, pixels).quality(ImageQuality::Medium);
    let effect = operation(
        OffsetInput::new().with_dx(0.75).with_dy(-1.25),
        domain,
        domain,
        domain,
    );
    let mut scene = Scene::new();
    scene.layer(
        Layer::new()
            .with_offset(effect)
            .with_transform(Transform::try_new([2.0, 1.0, -0.5, 1.5, 4.0, 3.0]).unwrap()),
        |child| {
            child.image(source, domain, ImageFit::Stretch);
        },
    );
    let output = render(&mut renderer, &mut surface, &scene);
    // At surface center(12.5,10.5), A^-1 gives p=(33/7,13/7).
    // q=p-(.75,-1.25); (qx+2qy)/20=285/560, or129.7768 bytes.
    assert_pixel(&output, 12, 10, [130, 0, 0, 255], 3);
}

#[test]
fn public_offset_rebased_image_sampling_retains_a_large_logical_origin() {
    let (mut renderer, mut surface) = renderer_and_surface(4, 1);
    let source = Rect::new(100_000_000.0, 0.0, 2.0, 1.0);
    let output_domain = Rect::new(100_000_000.0, 0.0, 4.0, 1.0);
    let effect = operation(
        OffsetInput::new().with_dx(0.5),
        source,
        output_domain,
        output_domain,
    );
    let mut scene = Scene::new();
    scene.layer(
        Layer::new()
            .with_offset(effect)
            .with_transform(Transform::translation(-100_000_000.0, 0.0).unwrap()),
        |child| {
            child.image(
                image(2, 1, vec![255, 0, 0, 255, 0, 0, 255, 255]),
                source,
                ImageFit::Stretch,
            );
        },
    );
    let output = render(&mut renderer, &mut surface, &scene);
    // Uploading this absolute origin would erase adjacent texel centers in
    // f32. The checked rebased texel map is simply j-.5 instead.
    assert_pixel(&output, 0, 0, [255, 0, 0, 128], 2);
    assert_pixel(&output, 1, 0, [128, 0, 128, 255], 2);
    assert_pixel(&output, 2, 0, [0, 0, 255, 128], 2);
    assert_pixel(&output, 3, 0, [0; 4], 1);
}

#[test]
fn public_offset_tiny_nonzero_inverse_map_refuses_without_replacing_publication() {
    let (mut renderer, mut surface) = renderer_and_surface(2, 1);
    let domain = bounds(2, 1);
    let source = image(2, 1, vec![255, 0, 0, 255, 0, 0, 255, 255]);
    let valid = operation(OffsetInput::new(), domain, domain, domain);
    let before = render(
        &mut renderer,
        &mut surface,
        &image_scene(source.clone(), domain, valid),
    );
    let stats = renderer.stats();
    let bad = operation(OffsetInput::new().with_dx(1.0e-300), domain, domain, domain);
    let error = pollster::block_on(renderer.render(
        &mut surface,
        &image_scene(source.clone(), domain, bad),
        Parameters::default(),
    ))
    .unwrap_err();
    assert!(matches!(
        error.offset_diagnostic(),
        Some(crate::OffsetPreparationError::Numeric {
            phase: crate::OffsetNumericPhase::GridMapping,
            ..
        })
    ));
    assert_eq!(renderer.stats(), stats);
    assert_eq!(
        pollster::block_on(renderer.read_headless(&surface)).unwrap(),
        before
    );
    assert_eq!(
        render(
            &mut renderer,
            &mut surface,
            &image_scene(source, domain, valid)
        ),
        before
    );
}

#[test]
fn public_offset_translates_same_layer_backdrop_after_restoring_input_lattice() {
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
    let effect = operation(OffsetInput::new().with_dx(0.5), source, full, full);
    let mut scene = Scene::new();
    scene.fill(
        Rect::new(1.0, 0.0, 1.0, 1.0),
        Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap(),
    );
    scene.layer(
        Layer::new()
            .with_offset(effect)
            .try_backdrop_filter(backdrop)
            .unwrap(),
        |_| {},
    );
    let output = render(&mut renderer, &mut surface, &scene);
    assert_pixel(&output, 2, 0, [255, 0, 0, 128], 2);
    assert_pixel(&output, 3, 0, [0; 4], 1);
}

#[test]
fn public_offset_noninteger_density_restores_negative_backdrop_lattice_before_translation() {
    use crate::{
        BackdropCaptureBounds, BackdropFilterInput, FilterBlur, FilterList, FilterOp, Options, Size,
    };
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(4.0, 2.0), 1.5)).unwrap();
    let full = Rect::new(0.0, 0.0, 4.0, 2.0);
    let source = Rect::new(-2.0, 0.0, 8.0 / 3.0, 2.0);
    let filter = Rect::new(-3.0, 0.0, 7.0, 2.0);
    let backdrop = BackdropFilterInput::try_new(
        FilterList::try_ops(vec![FilterOp::blur(FilterBlur::try_new(0.0).unwrap())]).unwrap(),
        BackdropCaptureBounds::try_new(Rect::new(-2.5, 0.0, 6.5, 2.0)).unwrap(),
        None,
    )
    .unwrap();
    let effect = operation(OffsetInput::new().with_dx(1.0 / 3.0), source, filter, full);
    let mut scene = Scene::new();
    scene.fill(
        Rect::new(0.0, 0.0, 2.0 / 3.0, 2.0),
        Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap(),
    );
    scene.layer(
        Layer::new()
            .with_offset(effect)
            .try_backdrop_filter(backdrop)
            .unwrap(),
        |_| {},
    );
    let output = render(&mut renderer, &mut surface, &scene);
    // rho1.5 gives backdrop origin floor(-2.5*1.5)=-4 and D origin-3.
    // Raw rebase must copy with integer delta1, then dx1/3 shifts one red
    // device cell byhalf a texel. Physical x1 was transparent, and receives.5.
    assert_pixel(&output, 1, 1, [255, 0, 0, 128], 2);
    assert_pixel(&output, 2, 1, [0; 4], 1);
    let samples = renderer
        .last_attempt()
        .unwrap()
        .attempt()
        .unwrap()
        .usage()
        .work(crate::RenderWork::ScheduledOffsetSamples);
    // D4x3 raw cells + O6x3 cells×4 bilinear opportunities =84.
    assert_eq!(samples, 84);
}

#[test]
fn public_offset_empty_domains_and_huge_disjoint_translation_preserve_transparent_composition() {
    use crate::{BlendMode, LayerComposition, PorterDuffOperator};
    let (mut renderer, mut surface) = renderer_and_surface(3, 1);
    let full = bounds(3, 1);
    for (source, result, dx) in [
        (Rect::new(0.0, 0.0, 0.0, 1.0e300), full, 0.0),
        (full, Rect::new(0.0, 0.0, 0.0, 1.0e300), 0.0),
        (full, full, 1.0e300),
        (full, full, -1.0e300),
    ] {
        let effect = operation(OffsetInput::new().with_dx(dx), source, full, result);
        let mut scene = Scene::new();
        scene.fill(full, Color::try_rgba(0.0, 1.0, 0.0, 1.0).unwrap());
        scene.layer(
            Layer::new().with_offset(effect).with_composition(
                LayerComposition::try_porter_duff(PorterDuffOperator::Copy, BlendMode::Normal)
                    .unwrap(),
            ),
            |child| {
                child.fill(full, Color::try_rgba(1.0, 1.0, 1.0, 1.0).unwrap());
            },
        );
        let output = render(&mut renderer, &mut surface, &scene);
        for x in 0..3 {
            assert_eq!(pixel(&output, x, 0), [0; 4]);
        }
    }
}

#[test]
fn public_offset_outer_clip_mask_and_opacity_keep_their_unshifted_frames() {
    use crate::{ResolvedLayerAlphaMask, Shape};
    let (mut renderer, mut surface) = renderer_and_surface(6, 1);
    let full = bounds(6, 1);
    let source = Rect::new(0.0, 0.0, 4.0, 1.0);
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
    let effect = operation(OffsetInput::new().with_dx(1.0), source, full, full);
    let layer = Layer::new()
        .with_offset(effect)
        .try_clip(Shape::rect(Rect::new(2.0, 0.0, 2.0, 1.0)))
        .unwrap()
        .with_resolved_alpha_mask(mask)
        .try_opacity(0.5)
        .unwrap();
    let mut scene = Scene::new();
    scene.layer(layer, |child| {
        child.fill(source, Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap());
    });
    let output = render(&mut renderer, &mut surface, &scene);
    // Offset input paints x1..5. The stationary clip retains x2/x3;
    // stationary mask alpha is128/255 atx2 and0 atx3; outer opacity is1/2.
    assert_pixel(&output, 2, 0, [255, 0, 0, 64], 2);
    for x in [0, 1, 3, 4, 5] {
        assert_pixel(&output, x, 0, [0; 4], 1);
    }
}

#[test]
fn public_offset_empty_input_or_output_excludes_even_a_huge_backdrop_capture() {
    use crate::{
        BackdropCaptureBounds, BackdropFilterInput, FilterList, FilterOp, UnitFilterAmount,
    };
    let (mut renderer, mut surface) = renderer_and_surface(3, 1);
    let full = bounds(3, 1);
    let empty = Rect::new(0.0, 0.0, 0.0, 1.0e300);
    for (source, result) in [(empty, full), (full, empty)] {
        let backdrop = BackdropFilterInput::try_new(
            FilterList::try_ops(vec![FilterOp::invert(
                UnitFilterAmount::try_new(1.0).unwrap(),
            )])
            .unwrap(),
            BackdropCaptureBounds::try_new(Rect::new(0.0, 0.0, 1.0e12, 1.0e12)).unwrap(),
            None,
        )
        .unwrap();
        let effect = operation(OffsetInput::new(), source, full, result);
        let mut scene = Scene::new();
        scene.fill(full, Color::try_rgba(0.0, 1.0, 0.0, 1.0).unwrap());
        scene.layer(
            Layer::new()
                .with_offset(effect)
                .try_backdrop_filter(backdrop)
                .unwrap(),
            |child| {
                child.fill(full, Color::try_rgba(1.0, 1.0, 1.0, 1.0).unwrap());
            },
        );
        let output = render(&mut renderer, &mut surface, &scene);
        // Attempting this excluded capture would exceed real device limits.
        // Success with unchanged green proves it is neither needed nor sampled.
        for x in 0..3 {
            assert_pixel(&output, x, 0, [0, 255, 0, 255], 1);
        }
        assert_eq!(
            renderer
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .work(crate::RenderWork::ScheduledOffsetSamples),
            0
        );
    }
}
