#![forbid(unsafe_code)]
#![cfg(not(target_family = "wasm"))]

use std::sync::{Arc, Weak};
use surgeist_render::{Color, Image, ImageFit, Layer, Rect, RenderAccountingContext, Scene, Size};

fn nested_scene(with_siblings: bool) -> (Scene, Weak<[u8]>) {
    let rect = Rect::try_new(0.0, 0.0, 1.0, 1.0).unwrap();
    let pixels = Arc::<[u8]>::from([255, 0, 0, 255]);
    let lifetime = Arc::downgrade(&pixels);
    let image = Image::from_rgba(Size::try_new(1.0, 1.0).unwrap(), pixels).unwrap();
    let mut scene = Scene::new();
    scene.image(image.clone(), rect, ImageFit::Fill);
    for _ in 0..8_192 {
        let child = scene;
        scene = Scene::new();
        if with_siblings {
            scene.image(image.clone(), rect, ImageFit::Fill);
        }
        scene.layer(Layer::new(), |target| *target = child);
        if with_siblings {
            scene.image(image.clone(), rect, ImageFit::Fill);
        }
    }
    (scene, lifetime)
}

fn on_bounded_stack(name: &str, run: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .name(name.into())
        .stack_size(256 * 1024)
        .spawn(run)
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn clearing_deep_scenes_releases_images_and_allows_reuse() {
    on_bounded_stack("clear deep authored scene", || {
        for with_siblings in [false, true] {
            let (mut scene, pixels) = nested_scene(with_siblings);
            assert!(pixels.upgrade().is_some());
            scene.clear();
            assert!(scene.is_empty());
            assert!(pixels.upgrade().is_none());

            scene.fill(Rect::try_new(0.0, 0.0, 2.0, 2.0).unwrap(), Color::BLACK);
            let summary = scene
                .inspect(&mut RenderAccountingContext::default())
                .unwrap();
            assert_eq!(summary.commands, 1);
            assert_eq!(summary.fills, 1);
        }
    });
}

#[test]
fn dropping_deep_scenes_releases_all_owned_images() {
    on_bounded_stack("drop deep authored scene", || {
        for with_siblings in [false, true] {
            let (scene, pixels) = nested_scene(with_siblings);
            assert!(pixels.upgrade().is_some());
            drop(scene);
            assert!(pixels.upgrade().is_none());
        }
    });
}
