#![cfg(not(target_arch = "wasm32"))]
#![forbid(unsafe_code)]

use surgeist_render::{
    Antialiasing, ClipInput, Color, CoordinateSpaceTag, EffectQualityPolicy, FillRule, FilledPath,
    Image, ImageBuffer, Layer, Options, Parameters, Path, Point, Rect, RenderRoute, Renderer,
    ResolvedLayerAlphaMask, Scene, Size, Transform,
};

fn clipped_scene(fill_rule: FillRule) -> Scene {
    let mut path = Path::new();
    // Both contours have the same winding: only EvenOdd cuts out the inner one.
    for (left, top, right, bottom) in [(-2.0, -1.0, 12.0, 9.0), (2.0, 2.0, 6.0, 6.0)] {
        path.move_to(Point::try_new(left, top).unwrap())
            .line_to(Point::try_new(right, top).unwrap())
            .line_to(Point::try_new(right, bottom).unwrap())
            .line_to(Point::try_new(left, bottom).unwrap())
            .close();
    }
    let clip = ClipInput::try_filled_path(FilledPath::try_new(path, fill_rule).unwrap())
        .unwrap()
        .with_coordinate_space(
            CoordinateSpaceTag::surface(Transform::translation(0.25, -0.25).unwrap()).unwrap(),
        );
    let source = Rect::try_new(-3.0, -2.0, 21.0, 16.0).unwrap();
    let opaque =
        Image::from_rgba(Size::try_new(1.0, 1.0).unwrap(), vec![255, 255, 255, 255]).unwrap();
    let mask = ResolvedLayerAlphaMask::try_new(opaque, source).unwrap();
    let mut scene = Scene::new();
    scene.layer(Layer::new().try_opacity(0.5).unwrap(), |scene| {
        scene.layer(
            Layer::new()
                .try_clip_input(clip)
                .unwrap()
                .with_resolved_alpha_mask(mask),
            |scene| {
                scene.fill(source, Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap());
            },
        );
    });
    scene
}

fn pixel(image: &ImageBuffer, x: u32, y: u32) -> [u8; 4] {
    let offset = usize::try_from((y * image.size().width() + x) * 4).unwrap();
    image.rgba()[offset..offset + 4].try_into().unwrap()
}

#[test]
fn signed_tagged_clip_preserves_fill_rule_and_masked_opacity_on_gpu_graph() {
    let mut renderer = pollster::block_on(Renderer::new(
        Options::default()
            .with_antialiasing(Antialiasing::Msaa16)
            .with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision),
    ))
    .unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::try_new(32.0, 24.0).unwrap(), 1.25))
            .unwrap();

    for fill_rule in [FillRule::EvenOdd, FillRule::NonZero] {
        let stats = pollster::block_on(renderer.render(
            &mut surface,
            &clipped_scene(fill_rule),
            Parameters::default(),
        ))
        .unwrap();
        assert_eq!(stats.route, Some(RenderRoute::GpuGraph));
        let image = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        assert_eq!((image.size().width(), image.size().height()), (40, 30));

        // The surface tag places the outer contour at logical
        // [-1.75,-1.25]..[12.25,8.75], and its hole at
        // [2.25,1.75]..[6.25,5.75]. At scale 5/4, each sampled pixel's
        // complete logical footprint lies strictly inside its asserted region:
        // (0,0): [0,0]..[0.8,0.8]; (11,5): [8.8,4]..[9.6,4.8];
        // (5,5): [4,4]..[4.8,4.8]; (20,6): [16,4.8]..[16.8,5.6].
        // The last remains inside the source and mask but outside the clip.
        // Opaque red through an opaque mask and opacity 0.5 reads back as
        // straight RGBA [255,0,0,128]; no edge-coverage tolerance is needed.
        for (x, y) in [(0, 0), (11, 5)] {
            assert_eq!(
                pixel(&image, x, y),
                [255, 0, 0, 128],
                "{fill_rule:?} ({x},{y})"
            );
        }
        let inner = match fill_rule {
            FillRule::EvenOdd => [0, 0, 0, 0],
            FillRule::NonZero => [255, 0, 0, 128],
        };
        assert_eq!(pixel(&image, 5, 5), inner, "{fill_rule:?} inner contour");
        assert_eq!(
            pixel(&image, 20, 6),
            [0, 0, 0, 0],
            "{fill_rule:?} outside clip"
        );
    }
}
