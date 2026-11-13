#![forbid(unsafe_code)]
#![cfg(not(target_family = "wasm"))]

use surgeist_render::{Color, Layer, Options, Parameters, Renderer, Scene, Size};

fn empty_layer_frame(depth: usize) {
    std::thread::Builder::new()
        .name("qualify nested empty layers".into())
        .stack_size(2 * 1024 * 1024)
        .spawn(move || {
            let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
            let mut surface =
                pollster::block_on(renderer.create_headless(Size::try_new(2.0, 2.0).unwrap(), 1.0))
                    .unwrap();
            let mut scene = Scene::new();
            for _ in 0..depth {
                let child = scene;
                scene = Scene::new();
                scene.layer(Layer::new(), |target| *target = child);
            }
            pollster::block_on(renderer.render(
                &mut surface,
                &scene,
                Parameters {
                    base_color: Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap(),
                    debug: false,
                },
            ))
            .unwrap();
            let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
            assert_eq!(pixels.rgba(), [255, 0, 0, 255].repeat(4));
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn shallow_empty_layers_preserve_base_color() {
    empty_layer_frame(2);
}

#[test]
fn deep_empty_layers_preserve_base_color() {
    empty_layer_frame(8_192);
}
