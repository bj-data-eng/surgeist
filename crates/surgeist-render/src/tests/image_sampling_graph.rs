#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

use crate::{
    BackdropCaptureBounds, BackdropFilterInput, ClipInput, Color, FilterList, FilterOp, Image,
    ImageBuffer, ImageFit, ImageRendering, Layer, Options, Paint, Parameters, Point, Rect,
    RenderRoute, Renderer, ResolvedLayerAlphaMask, Scene, Shape, Size, Transform, UnitFilterAmount,
};

fn color(rgba: [u8; 4]) -> Color {
    Color::try_rgba(
        f32::from(rgba[0]) / 255.0,
        f32::from(rgba[1]) / 255.0,
        f32::from(rgba[2]) / 255.0,
        f32::from(rgba[3]) / 255.0,
    )
    .unwrap()
}

fn image(hint: ImageRendering) -> Image {
    Image::from_rgba(
        Size::new(2.0, 1.0),
        Arc::<[u8]>::from([0, 0, 0, 255, 255, 255, 255, 255]),
    )
    .unwrap()
    .rendering(hint)
}

fn masked_layer(bounds: Rect) -> Layer {
    Layer::new().with_resolved_alpha_mask(
        ResolvedLayerAlphaMask::try_new(
            Image::from_rgba(Size::new(1.0, 1.0), Arc::<[u8]>::from([0, 0, 0, 128])).unwrap(),
            bounds,
        )
        .unwrap(),
    )
}

fn render(renderer: &mut Renderer, scene: &Scene, size: Size, dpr: f64) -> ImageBuffer {
    let mut surface = pollster::block_on(renderer.create_headless(size, dpr)).unwrap();
    let stats =
        pollster::block_on(renderer.render(&mut surface, scene, Parameters::default())).unwrap();
    assert_eq!(stats.route, Some(RenderRoute::GpuGraph));
    pollster::block_on(renderer.read_headless(&surface)).unwrap()
}

fn assert_pixel(output: &ImageBuffer, x: u32, y: u32, expected: [u8; 4]) {
    let offset = usize::try_from((y * output.size().width() + x) * 4).unwrap();
    let observed = &output.rgba()[offset..offset + 4];
    // The established RGBA8/half-float GPU boundary allows two byte levels.
    for (observed, expected) in observed.iter().copied().zip(expected) {
        assert!(
            observed.abs_diff(expected) <= 2,
            "pixel ({x}, {y}) {observed} != {expected}"
        );
    }
}

#[test]
fn graph_draws_and_image_paints_preserve_every_hint_at_fractional_and_integer_scales() {
    let mut scene = Scene::new();
    let mut expected_rows = Vec::new();
    scene.layer(masked_layer(Rect::new(0.0, 0.0, 6.0, 20.0)), |scene| {
        for (hint, fractional, integer) in [
            (
                ImageRendering::Auto,
                [0, 26, 128, 230, 255],
                [0, 64, 191, 255],
            ),
            (
                ImageRendering::Smooth,
                [0, 26, 128, 230, 255],
                [0, 64, 191, 255],
            ),
            // Mitchell's padded step response is 131/1125 at x=0.1 and
            // 139/576 at x=0.25: byte values 30 and 62, respectively.
            (
                ImageRendering::HighQuality,
                [0, 30, 128, 225, 255],
                [0, 62, 193, 255],
            ),
            (
                ImageRendering::CrispEdges,
                [0, 0, 255, 255, 255],
                [0, 0, 255, 255],
            ),
            (
                ImageRendering::Pixelated,
                [0, 0, 128, 255, 255],
                [0, 0, 255, 255],
            ),
        ] {
            for paint in [false, true] {
                for values in [fractional.to_vec(), integer.to_vec()] {
                    let y = f64::from(u32::try_from(expected_rows.len()).unwrap());
                    let width = f64::from(u32::try_from(values.len()).unwrap());
                    if paint {
                        scene.layer(
                            Layer::new().with_transform(
                                Transform::try_new([width / 2.0, 0.0, 0.0, 1.0, 0.0, y]).unwrap(),
                            ),
                            |scene| {
                                scene
                                    .fill(Rect::new(0.0, 0.0, 2.0, 1.0), Paint::image(image(hint)));
                            },
                        );
                    } else {
                        scene.image(image(hint), Rect::new(0.0, y, width, 1.0), ImageFit::Fill);
                    }
                    expected_rows.push(values);
                }
            }
        }
    });
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let output = render(&mut renderer, &scene, Size::new(6.0, 20.0), 1.0);
    for (y, row) in expected_rows.into_iter().enumerate() {
        for (x, value) in row.iter().copied().enumerate() {
            assert_pixel(&output, x as u32, y as u32, [value, value, value, 128]);
        }
        for x in u32::try_from(row.len()).unwrap()..6 {
            assert_pixel(&output, x, y as u32, [0; 4]);
        }
    }
}

#[test]
fn graph_sensitive_branch_keeps_nested_clip_mask_opacity_and_ordinary_siblings() {
    let mut scene = Scene::new();
    scene.layer(
        masked_layer(Rect::new(0.0, 0.0, 2.0, 2.0))
            .with_transform(Transform::try_new([2.5, 0.0, 0.0, 2.0, 1.0, 1.0]).unwrap())
            .try_clip(Shape::try_circle(Point::new(1.0, 1.0), 0.8).unwrap())
            .unwrap(),
        |scene| {
            scene.layer(Layer::new().try_opacity(0.5).unwrap(), |scene| {
                scene.image(
                    image(ImageRendering::Pixelated),
                    Rect::new(0.0, 0.0, 2.0, 2.0),
                    ImageFit::Fill,
                );
            });
        },
    );
    scene.layer(
        Layer::new()
            .with_transform(Transform::translation(8.0, 2.0).unwrap())
            .try_opacity(0.5)
            .unwrap(),
        |scene| {
            scene.layer(
                Layer::new().with_transform(Transform::scale(2.0, 1.0).unwrap()),
                |scene| {
                    scene.fill(
                        Rect::new(0.0, 0.0, 2.0, 1.0),
                        Paint::image(image(ImageRendering::Smooth)),
                    );
                },
            );
        },
    );
    scene.fill(Rect::new(10.0, 2.0, 1.0, 1.0), color([255, 0, 0, 255]));
    scene.fill(Rect::new(8.0, 5.0, 4.0, 1.0), color([0, 255, 0, 255]));
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let output = render(&mut renderer, &scene, Size::new(14.0, 8.0), 1.0);
    for (x, value) in [(2, 0), (3, 128), (4, 255)] {
        assert_pixel(&output, x, 2, [value, value, value, 64]);
    }
    assert_pixel(&output, 0, 2, [0; 4]);
    assert_pixel(&output, 3, 0, [0; 4]);
    // This pixel is inside the image and mask rectangles, but the circle
    // covers only a corner of it; a rectangular clip would leave alpha 64.
    let clipped_alpha =
        output.rgba()[usize::try_from((output.size().width() + 1) * 4 + 3).unwrap()];
    assert!(clipped_alpha < 64);
    assert_pixel(&output, 8, 2, [0, 0, 0, 128]);
    assert_pixel(&output, 9, 2, [64, 64, 64, 128]);
    assert_pixel(&output, 10, 2, [255, 0, 0, 255]);
    assert_pixel(&output, 11, 2, [255, 255, 255, 128]);
    assert_pixel(&output, 9, 5, [0, 255, 0, 255]);
}

#[test]
fn graph_sampling_uses_full_affine_device_coordinates_and_unclipped_source_origins() {
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    for hint in [ImageRendering::Pixelated, ImageRendering::CrispEdges] {
        let pixelated = hint == ImageRendering::Pixelated;
        // These source coordinates follow by inverting each explicit affine.
        // Pixelated uses three virtual columns per original source column;
        // coordinate 1.1 blends the virtual step by 0.8 (alpha-independent 204).
        let cases = [
            (
                Transform::try_new([0.0, 2.5, -1.0, 0.0, 3.0, 1.0]).unwrap(),
                Rect::new(0.0, 0.0, 2.0, 2.0),
                1.0,
                vec![
                    (1, 1, 0),
                    (1, 2, 0),
                    (1, 3, if pixelated { 128 } else { 255 }),
                    (1, 4, 255),
                    (1, 5, 255),
                ],
            ),
            (
                Transform::try_new([2.5, 0.0, 1.0, 2.0, 1.0, 1.0]).unwrap(),
                Rect::new(0.0, 0.0, 2.0, 2.0),
                1.0,
                vec![
                    (3, 2, 0),
                    (4, 2, if pixelated { 204 } else { 255 }),
                    (5, 2, 255),
                ],
            ),
            (
                Transform::try_new([2.5, 0.0, 0.0, 2.0, 0.25, 1.0]).unwrap(),
                Rect::new(0.0, 0.0, 2.0, 2.0),
                1.0,
                vec![
                    (1, 2, 0),
                    (2, 2, if pixelated { 51 } else { 0 }),
                    (3, 2, 255),
                    (4, 2, 255),
                ],
            ),
            (
                Transform::try_new([2.5, 0.0, 0.0, 2.0, 2.5, 2.0]).unwrap(),
                Rect::new(-1.0, -1.0, 2.0, 2.0),
                1.0,
                vec![
                    (0, 1, 0),
                    (1, 1, 0),
                    (2, 1, if pixelated { 128 } else { 255 }),
                    (3, 1, 255),
                    (4, 1, 255),
                ],
            ),
            (
                Transform::try_new([2.5, 0.0, 0.0, 2.0, -1.25, -2.0]).unwrap(),
                Rect::new(0.0, 0.0, 2.0, 2.0),
                1.0,
                vec![
                    (0, 0, 0),
                    (1, 0, if pixelated { 204 } else { 255 }),
                    (2, 0, 255),
                ],
            ),
            (
                Transform::try_new([1.25, 0.0, 0.0, 1.0, 0.5, 0.5]).unwrap(),
                Rect::new(0.0, 0.0, 2.0, 2.0),
                2.0,
                vec![
                    (1, 2, 0),
                    (2, 2, 0),
                    (3, 2, if pixelated { 128 } else { 255 }),
                    (4, 2, 255),
                    (5, 2, 255),
                ],
            ),
        ];
        for (transform, bounds, dpr, expected) in cases {
            let mut scene = Scene::new();
            scene.layer(masked_layer(Rect::new(-16.0, -16.0, 32.0, 32.0)), |scene| {
                scene.layer(Layer::new().with_transform(transform), |scene| {
                    scene.image(image(hint), bounds, ImageFit::Fill);
                });
            });
            let output = render(&mut renderer, &scene, Size::new(8.0, 8.0), dpr);
            for (x, y, value) in expected {
                assert_pixel(&output, x, y, [value, value, value, 128]);
            }
        }
    }
}

#[test]
fn graph_backdrop_filters_only_its_clip_before_crisp_foreground_and_later_siblings() {
    let base = [51, 102, 153, 255];
    let mut scene = Scene::new();
    scene.fill(Rect::new(0.0, 0.0, 7.0, 3.0), color(base));
    let filters = FilterList::try_ops(vec![FilterOp::invert(
        UnitFilterAmount::try_new(1.0).unwrap(),
    )])
    .unwrap();
    scene.layer(
        Layer::new()
            .try_backdrop_filter(
                BackdropFilterInput::try_new(
                    filters,
                    BackdropCaptureBounds::try_new(Rect::new(1.0, 0.0, 5.0, 3.0)).unwrap(),
                    Some(ClipInput::try_shape(Shape::rect(Rect::new(2.0, 0.0, 3.0, 3.0))).unwrap()),
                )
                .unwrap(),
            )
            .unwrap(),
        |scene| {
            let foreground = Image::from_rgba(
                Size::new(2.0, 1.0),
                Arc::<[u8]>::from([255, 0, 0, 255, 0, 0, 255, 255]),
            )
            .unwrap()
            .rendering(ImageRendering::CrispEdges);
            scene.image(foreground, Rect::new(1.0, 1.0, 5.0, 1.0), ImageFit::Fill);
        },
    );
    scene.fill(Rect::new(4.0, 1.0, 1.0, 1.0), color([0, 255, 0, 255]));
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let output = render(&mut renderer, &scene, Size::new(7.0, 3.0), 1.0);
    for y in [0, 2] {
        for x in 0..7 {
            assert_pixel(
                &output,
                x,
                y,
                if (2..5).contains(&x) {
                    [204, 153, 102, 255]
                } else {
                    base
                },
            );
        }
    }
    for (x, expected) in [
        base,
        [255, 0, 0, 255],
        [255, 0, 0, 255],
        [0, 0, 255, 255],
        [0, 255, 0, 255],
        [0, 0, 255, 255],
        base,
    ]
    .into_iter()
    .enumerate()
    {
        assert_pixel(&output, x as u32, 1, expected);
    }
}
