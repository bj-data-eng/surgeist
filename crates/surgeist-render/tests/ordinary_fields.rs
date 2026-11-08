#![cfg(not(target_arch = "wasm32"))]
#![forbid(unsafe_code)]

use surgeist_render::{
    Color, Gradient, GradientStop, ImageBuffer, Options, Parameters, Point, Rect, RenderRoute,
    Renderer, Scene, Size, Transform,
};

fn pixel(image: &ImageBuffer, x: u32, y: u32) -> [u8; 4] {
    let index = usize::try_from((y * image.size().width() + x) * 4).unwrap();
    image.rgba()[index..index + 4].try_into().unwrap()
}

#[test]
fn linear_gradient_preserves_its_local_field_under_shear() {
    let gradient = Gradient::try_linear(
        Point::try_new(0.0, 0.0).unwrap(),
        Point::try_new(8.0, 0.0).unwrap(),
        vec![
            GradientStop::try_new(0.0, Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap()).unwrap(),
            GradientStop::try_new(1.0, Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap()).unwrap(),
        ],
    )
    .unwrap();
    let mut scene = Scene::new();
    scene.transform(
        Transform::try_new([1.0, 0.0, 1.0, 1.0, 0.0, 0.0]).unwrap(),
        |scene| {
            scene.fill(Rect::try_new(0.0, 0.0, 8.0, 8.0).unwrap(), gradient);
        },
    );
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::try_new(16.0, 8.0).unwrap(), 1.0))
            .unwrap();
    let stats =
        pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default())).unwrap();
    assert_eq!(stats.route, Some(RenderRoute::DirectVello));
    let image = pollster::block_on(renderer.read_headless(&surface)).unwrap();

    // T(x,y)=(x+y,y), so T^-1(x',y')=(x'-y',y'). The local line
    // (0,0)->(8,0) therefore has q=(x'-y')/8. At these fully covered
    // integer physical samples q is 1/4, 1/2, 3/4, and 1/4. The opaque
    // endpoints have premultiplied RGBA (1-q,0,q,1), giving these rounded
    // bytes independently of any transformed endpoint or native paint map.
    let samples = [
        (4, 2, [191, 0, 64, 255]),
        (6, 2, [128, 0, 128, 255]),
        (8, 2, [64, 0, 191, 255]),
        (6, 4, [191, 0, 64, 255]),
    ];
    for &(x, y, expected) in &samples {
        println!(
            "sheared linear ({x},{y}): actual {:?}, expected {expected:?}",
            pixel(&image, x, y)
        );
    }
    for (x, y, expected) in samples {
        let actual = pixel(&image, x, y);
        assert!(
            actual
                .into_iter()
                .zip(expected)
                .all(|(actual, expected)| actual.abs_diff(expected) <= 2),
            "sheared linear ({x},{y}): actual {actual:?}, expected {expected:?}, tolerance 2"
        );
    }
}
