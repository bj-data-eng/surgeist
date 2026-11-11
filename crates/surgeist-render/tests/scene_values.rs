#![forbid(unsafe_code)]

use std::fmt::{self, Write};
#[cfg(not(target_family = "wasm"))]
use std::sync::Arc;
use surgeist_render::{Color, Layer, Rect, Scene};
#[cfg(not(target_family = "wasm"))]
use surgeist_render::{Image, ImageFit, RenderAccountingContext, Size};

fn rect(x: f64) -> Rect {
    Rect::try_new(x, 0.0, 1.0, 1.0).unwrap()
}

fn nest(mut scene: Scene, depth: u64, siblings: bool) -> Scene {
    for _ in 0..depth {
        let child = scene;
        scene = Scene::new();
        scene.layer(Layer::new(), |target| *target = child);
        if siblings {
            scene.fill(rect(2.0), Color::TRANSPARENT);
        }
    }
    scene
}

fn filled(color: Color) -> Scene {
    let mut scene = Scene::new();
    scene.fill(rect(0.0), color);
    scene
}

#[cfg(not(target_family = "wasm"))]
const DEPTH: u64 = 8_192;

#[cfg(not(target_family = "wasm"))]
fn on_bounded_stack(name: &str, run: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .name(name.into())
        .stack_size(256 * 1024)
        .spawn(run)
        .unwrap()
        .join()
        .unwrap();
}

#[cfg(not(target_family = "wasm"))]
#[test]
fn cloning_deep_scenes_preserves_content_and_independent_ownership() {
    on_bounded_stack("clone deep authored scene", || {
        for siblings in [false, true] {
            let pixels = Arc::<[u8]>::from([1, 2, 3, 255]);
            let lifetime = Arc::downgrade(&pixels);
            let image = Image::from_rgba(Size::try_new(1.0, 1.0).unwrap(), pixels).unwrap();
            let mut leaf = Scene::new();
            leaf.image(image, rect(0.0), ImageFit::Fill);
            let mut original = nest(leaf, DEPTH, siblings);
            let mut copy = original.clone();
            assert!(copy == original);
            original.clear();
            assert!(lifetime.upgrade().is_some());
            let summary = copy
                .inspect(&mut RenderAccountingContext::default())
                .unwrap();
            assert_eq!(summary.layers, DEPTH);
            assert_eq!(summary.images, 1);
            assert_eq!(summary.fills, u64::from(siblings) * DEPTH);
            assert_eq!(summary.commands, summary.layers + summary.fills + 1);
            assert!(original.is_empty());
            copy.clear();
            assert!(lifetime.upgrade().is_none());
        }
    });
}

#[cfg(not(target_family = "wasm"))]
#[test]
fn comparing_deep_scenes_checks_actual_leaves() {
    on_bounded_stack("compare deep authored scenes", || {
        for siblings in [false, true] {
            let scene = nest(filled(Color::BLACK), DEPTH, siblings);
            assert!(scene == nest(filled(Color::BLACK), DEPTH, siblings));
            assert!(scene != nest(filled(Color::TRANSPARENT), DEPTH, siblings));
        }
    });
}

#[cfg(not(target_family = "wasm"))]
#[test]
fn formatting_deep_scenes_writes_all_layers_and_leaves() {
    on_bounded_stack("format deep authored scene", || {
        for siblings in [false, true] {
            let scene = nest(filled(Color::BLACK), DEPTH, siblings);
            let compact = format!("{scene:?}");
            assert_eq!(compact.matches("Layer { layer:").count(), DEPTH as usize);
            assert_eq!(
                compact.matches("Fill { shape:").count(),
                (1 + u64::from(siblings) * DEPTH) as usize
            );
        }

        // Pretty output itself grows with the indentation, so use a shorter
        // chain and check the leaf's full indentation independently.
        let pretty_depth = 256;
        let scene = nest(filled(Color::BLACK), pretty_depth, false);
        let pretty = format!("{scene:#?}");
        let leaf = pretty
            .lines()
            .find(|line| line.trim_start() == "Fill {")
            .unwrap();
        assert_eq!(
            leaf.len() - leaf.trim_start().len(),
            (2 + 2 * pretty_depth as usize) * 4
        );
    });
}

#[test]
fn equality_distinguishes_order_and_layer_boundaries() {
    let mut left = Scene::new();
    left.layer(Layer::new(), |child| {
        child.fill(rect(0.0), Color::BLACK);
        child.fill(rect(2.0), Color::TRANSPARENT);
    });
    let mut reordered = Scene::new();
    reordered.layer(Layer::new(), |child| {
        child.fill(rect(2.0), Color::TRANSPARENT);
        child.fill(rect(0.0), Color::BLACK);
    });
    let mut isolated = Scene::new();
    isolated.layer(Layer::new().isolated(), |child| {
        child.fill(rect(0.0), Color::BLACK);
        child.fill(rect(2.0), Color::TRANSPARENT);
    });
    let mut split = Scene::new();
    split.layer(Layer::new(), |child| {
        child.fill(rect(0.0), Color::BLACK);
    });
    split.fill(rect(2.0), Color::TRANSPARENT);
    assert!(left != reordered);
    assert!(left != isolated);
    assert!(left != split);
}

#[test]
fn ordinary_debug_preserves_derived_structure_and_writer_errors() {
    // Use Rust's derived formatting of a small independent authored tree.
    mod expected {
        use surgeist_render::{Layer, Paint, Shape};

        #[derive(Debug)]
        #[allow(dead_code)]
        pub(super) struct Scene {
            pub(super) commands: Vec<Command>,
        }

        #[derive(Debug)]
        #[allow(dead_code)]
        pub(super) enum Command {
            Fill {
                shape: Shape,
                paint: Paint,
            },
            Layer {
                layer: Box<Layer>,
                children: Vec<Self>,
            },
        }
    }
    let mut nested = filled(Color::BLACK);
    nested.layer(Layer::new(), |_| {});
    let mut scene = nest(nested, 2, true);
    scene.layer(Layer::new(), |_| {});
    let reference = expected::Scene {
        commands: vec![
            expected::Command::Layer {
                layer: Box::new(Layer::new()),
                children: vec![
                    expected::Command::Layer {
                        layer: Box::new(Layer::new()),
                        children: vec![
                            expected::Command::Fill {
                                shape: rect(0.0).into(),
                                paint: Color::BLACK.into(),
                            },
                            expected::Command::Layer {
                                layer: Box::new(Layer::new()),
                                children: Vec::new(),
                            },
                        ],
                    },
                    expected::Command::Fill {
                        shape: rect(2.0).into(),
                        paint: Color::TRANSPARENT.into(),
                    },
                ],
            },
            expected::Command::Fill {
                shape: rect(2.0).into(),
                paint: Color::TRANSPARENT.into(),
            },
            expected::Command::Layer {
                layer: Box::new(Layer::new()),
                children: Vec::new(),
            },
        ],
    };
    assert_eq!(format!("{scene:?}"), format!("{reference:?}"));
    let pretty = format!("{scene:#?}");
    assert_eq!(pretty, format!("{reference:#?}"));

    struct ShortWriter(usize);
    impl Write for ShortWriter {
        fn write_str(&mut self, text: &str) -> fmt::Result {
            self.0 = self.0.checked_sub(text.len()).ok_or(fmt::Error)?;
            Ok(())
        }
    }
    let mut writer = ShortWriter(pretty.find("Fill {").unwrap() + 4);
    assert_eq!(write!(&mut writer, "{scene:#?}"), Err(fmt::Error));
}
