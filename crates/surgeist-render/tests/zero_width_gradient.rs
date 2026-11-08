#![cfg(all(feature = "render-paint", not(target_arch = "wasm32")))]
#![forbid(unsafe_code)]

use surgeist_layout as layout;
use surgeist_paint as paint;
use surgeist_render::{
    Color, EffectQualityPolicy, Image, ImageBuffer, Layer, Options, Paint, Parameters, Rect,
    RenderAccountingContext, RenderRoute, Renderer, ResolvedLayerAlphaMask, Scene, Size, Surface,
    prepare_radial_gradient, prepare_repeating_radial_gradient,
};

fn options() -> paint::PaintOptions {
    paint::PaintOptions::new(paint::PaintLimits::unlimited(), paint::ProfileMode::Off)
}

fn position(length: f64, fraction: f64) -> paint::GradientPosition {
    paint::GradientPosition { length, fraction }
}

fn field(repeating: bool, height: f64, center_y: f64) -> Paint {
    let geometry = paint::resolve_radial_gradient(
        layout::Size::new(16.0, 8.0),
        Some(layout::Point::new(8.0, center_y)),
        paint::RadialGradientSize::EllipseRadii(layout::Size::new(
            position(0.0, 0.0),
            position(height, 0.0),
        )),
        options(),
    )
    .result
    .expect("authentic Paint radial geometry preparation must succeed");
    assert_eq!(geometry.case(), paint::RadialGradientCase::ZeroWidth);
    assert_eq!(geometry.center(), layout::Point::new(8.0, center_y));
    let (first, last) = if repeating { (1.0, 5.0) } else { (2.0, 6.0) };
    let red = Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap();
    let blue = Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap();
    let source = [
        paint::GradientStopItem::Stop {
            payload: red,
            positions: paint::GradientStopPositions::One(position(first, -1.0)),
        },
        paint::GradientStopItem::Stop {
            payload: blue,
            positions: paint::GradientStopPositions::One(position(last, 1.0)),
        },
    ];
    let used = paint::resolve_gradient_stops(0.0, &source, options())
        .result
        .expect("authentic Paint basis-zero used-stop preparation must succeed");
    assert_eq!(used.line_length(), 0.0);
    assert!(std::ptr::eq(used.source(), source.as_slice()));
    let records: Vec<_> = used
        .items()
        .map(|item| {
            let paint::UsedGradientItem::Stop {
                source_index,
                position_index,
                position,
                payload,
            } = item
            else {
                panic!("the two supplied colors must remain actual stop occurrences")
            };
            (source_index, position_index, position, *payload)
        })
        .collect();
    // Percentage contributions vanish at the authentic zero basis; finite
    // physical lengths and exact source associations remain unchanged.
    assert_eq!(records, [(0, 0, first, red), (1, 0, last, blue)]);
    let mut context = RenderAccountingContext::default();
    if repeating {
        let field = prepare_repeating_radial_gradient(&geometry, &used, &mut context)
            .expect("repeating radial consumer must accept authentic ZeroWidth with basis zero");
        assert_eq!(
            field
                .stops()
                .iter()
                .map(|stop| (stop.position(), stop.color()))
                .collect::<Vec<_>>(),
            [(first, red), (last, blue)]
        );
        field.into()
    } else {
        let field = prepare_radial_gradient(&geometry, &used, &mut context)
            .expect("ordinary radial consumer must accept authentic ZeroWidth with basis zero");
        assert_eq!(
            field
                .stops()
                .iter()
                .map(|stop| (stop.position(), stop.color()))
                .collect::<Vec<_>>(),
            [(first, red), (last, blue)]
        );
        field.into()
    }
}

fn bounds() -> Rect {
    Rect::try_new(0.0, 0.0, 16.0, 8.0).unwrap()
}

fn renderer_and_surface() -> (Renderer, Surface) {
    let mut renderer = pollster::block_on(Renderer::new(
        Options::default().with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision),
    ))
    .unwrap();
    let surface =
        pollster::block_on(renderer.create_headless(Size::try_new(16.0, 8.0).unwrap(), 1.0))
            .unwrap();
    (renderer, surface)
}

fn read(renderer: &mut Renderer, surface: &mut Surface, field: Paint, graph: bool) -> ImageBuffer {
    let mut scene = Scene::new();
    if graph {
        let mask =
            Image::from_rgba(Size::try_new(1.0, 1.0).unwrap(), vec![255, 255, 255, 128]).unwrap();
        scene.layer(
            Layer::new()
                .with_resolved_alpha_mask(ResolvedLayerAlphaMask::try_new(mask, bounds()).unwrap()),
            |scene| {
                scene.fill(bounds(), field);
            },
        );
    } else {
        scene.fill(bounds(), field);
    }
    let stats =
        pollster::block_on(renderer.render(surface, &scene, Parameters::default())).unwrap();
    assert_eq!(
        stats.route,
        Some(if graph {
            RenderRoute::GpuGraph
        } else {
            RenderRoute::DirectVello
        })
    );
    pollster::block_on(renderer.read_headless(surface)).unwrap()
}

fn assert_pixel(image: &ImageBuffer, x: u32, y: u32, rgb: [u8; 3], graph: bool) {
    let index = usize::try_from((y * image.size().width() + x) * 4).unwrap();
    let actual: [u8; 4] = image.rgba()[index..index + 4].try_into().unwrap();
    let expected = [rgb[0], rgb[1], rgb[2], if graph { 128 } else { 255 }];
    let tolerance = if graph { 3 } else { 2 };
    assert!(
        actual
            .into_iter()
            .zip(expected)
            .all(|(actual, expected)| actual.abs_diff(expected) <= tolerance),
        "zero-width pixel({x},{y}), graph={graph}: actual {actual:?}, expected {expected:?}, tolerance {tolerance}"
    );
}

#[test]
fn ordinary_zero_width_consumer_folds_distance_before_offset_and_pads_both_sides() {
    // These owned fields must survive all geometry, used/source storage and
    // contexts in field(). Neither GPU creation nor readback precedes acceptance.
    let fields = [field(false, 8.0, 4.0), field(false, 0.0, -100.0)];
    let (mut renderer, mut surface) = renderer_and_surface();
    for field in fields {
        for graph in [false, true] {
            let image = read(&mut renderer, &mut surface, field.clone(), graph);
            // r=abs(x-8), then t=clamp((r-2)/4,0,1). Opaque red/blue
            // premultiplied interpolation is (1-t,0,t,1). Folding after the
            // offset would incorrectly change both the center and left half.
            for (x, rgb) in [
                (8, [255, 0, 0]),
                (7, [255, 0, 0]),
                (9, [255, 0, 0]),
                (4, [128, 0, 128]),
                (12, [128, 0, 128]),
                (2, [0, 0, 255]),
                (14, [0, 0, 255]),
            ] {
                for y in [2, 6] {
                    assert_pixel(&image, x, y, rgb, graph);
                }
            }
        }
    }
}

#[test]
fn repeating_zero_width_consumer_folds_before_phase_and_preserves_both_seams() {
    let fields = [field(true, 8.0, 4.0), field(true, 0.0, -100.0)];
    let (mut renderer, mut surface) = renderer_and_surface();
    for field in fields {
        for graph in [false, true] {
            let image = read(&mut renderer, &mut surface, field.clone(), graph);
            // Physical first=1, last=5, T=4. Fold r=abs(x-8) before
            // u=fract((r-1)/4). Center u=.75; r1/r5 are exact red seams;
            // r2/r3/r4 have blue weights .25/.5/.75 on either half.
            for (x, rgb) in [
                (8, [64, 0, 191]),
                (7, [255, 0, 0]),
                (9, [255, 0, 0]),
                (6, [191, 0, 64]),
                (10, [191, 0, 64]),
                (5, [128, 0, 128]),
                (11, [128, 0, 128]),
                (4, [64, 0, 191]),
                (12, [64, 0, 191]),
                (3, [255, 0, 0]),
                (13, [255, 0, 0]),
            ] {
                for y in [2, 6] {
                    assert_pixel(&image, x, y, rgb, graph);
                }
            }
        }
    }
}
