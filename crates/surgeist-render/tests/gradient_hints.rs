#![cfg(all(feature = "render-paint", not(target_arch = "wasm32")))]
#![forbid(unsafe_code)]

use surgeist_layout as layout;
use surgeist_paint as paint;
use surgeist_render::{
    Color, EffectQualityPolicy, Gradient, Image, ImageBuffer, Layer, Options, Paint, Parameters,
    Rect, RenderAccountingContext, RenderRoute, Renderer, RepeatingGradient,
    ResolvedLayerAlphaMask, Scene, Size, Surface, prepare_linear_gradient, prepare_radial_gradient,
    prepare_repeating_linear_gradient, prepare_repeating_radial_gradient,
};

fn options() -> paint::PaintOptions {
    paint::PaintOptions::new(paint::PaintLimits::unlimited(), paint::ProfileMode::Off)
}

fn position(length: f64) -> paint::GradientPosition {
    paint::GradientPosition {
        length,
        fraction: 0.0,
    }
}

fn source(length: f64, alpha: f32) -> [paint::GradientStopItem<Color>; 3] {
    [
        paint::GradientStopItem::Stop {
            payload: Color::try_rgba(1.0, 0.0, 0.0, alpha).unwrap(),
            positions: paint::GradientStopPositions::One(position(0.0)),
        },
        paint::GradientStopItem::Hint {
            position: position(length / 4.0),
        },
        paint::GradientStopItem::Stop {
            payload: Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap(),
            positions: paint::GradientStopPositions::One(position(length)),
        },
    ]
}

fn line(length: f64) -> paint::LinearGradientLine {
    paint::resolve_linear_gradient(
        layout::Size::new(length, 16.0),
        paint::LinearGradientDirection::Right,
        options(),
    )
    .result
    .expect("authentic Paint line preparation must succeed")
}

fn used<'a>(
    basis: f64,
    source: &'a [paint::GradientStopItem<Color>],
) -> paint::GradientStops<'a, Color> {
    let used = paint::resolve_gradient_stops(basis, source, options())
        .result
        .expect("authentic Paint used-stop preparation must succeed");
    // Verify that the real producer retained the stimulus. The behavioral
    // assertion remains consumer acceptance followed by independently known pixels.
    assert!(
        used.items()
            .any(|item| matches!(item, paint::UsedGradientItem::Hint { .. }))
    );
    used
}

fn ordinary_line(alpha: f32) -> Gradient {
    let geometry = line(16.0);
    let source = source(16.0, alpha);
    let used = used(16.0, &source);
    let mut context = RenderAccountingContext::default();
    prepare_linear_gradient(&geometry, &used, &mut context)
        .expect("ordinary linear consumer must accept the authentic transition hint")
}

fn repeating_line(length: f64, alpha: f32) -> RepeatingGradient {
    let geometry = line(length);
    let source = source(length, alpha);
    let used = used(length, &source);
    let mut context = RenderAccountingContext::default();
    prepare_repeating_linear_gradient(&geometry, &used, &mut context)
        .expect("repeating linear consumer must accept the authentic transition hint")
}

fn rect() -> Rect {
    Rect::try_new(0.0, 0.0, 32.0, 16.0).unwrap()
}

fn renderer_and_surface() -> (Renderer, Surface) {
    let mut renderer = pollster::block_on(Renderer::new(
        Options::default().with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision),
    ))
    .unwrap();
    let surface =
        pollster::block_on(renderer.create_headless(Size::try_new(32.0, 16.0).unwrap(), 1.0))
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
                .with_resolved_alpha_mask(ResolvedLayerAlphaMask::try_new(mask, rect()).unwrap()),
            |scene| {
                scene.fill(rect(), field);
            },
        );
    } else {
        scene.fill(rect(), field);
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

fn assert_pixel(image: &ImageBuffer, x: u32, y: u32, mut expected: [u8; 4], graph: bool) {
    if graph {
        expected[3] = u8::try_from((u16::from(expected[3]) * 128 + 127) / 255).unwrap();
    }
    let index = usize::try_from((y * image.size().width() + x) * 4).unwrap();
    let actual: [u8; 4] = image.rgba()[index..index + 4].try_into().unwrap();
    let tolerance = if graph { 3 } else { 2 };
    assert!(
        actual
            .into_iter()
            .zip(expected)
            .all(|(a, e)| a.abs_diff(e) <= tolerance),
        "hint pixel ({x},{y}), graph={graph}: actual {actual:?}, expected {expected:?}, tolerance {tolerance}"
    );
}

#[test]
fn authentic_linear_hint_controls_half_and_three_quarter_premultiplied_colors() {
    // Each owned output survives the geometry, source array and context in
    // ordinary_line. No GPU is created until both consumers have accepted.
    let fields = [(1.0, ordinary_line(1.0)), (0.5, ordinary_line(0.5))];
    let (mut renderer, mut surface) = renderer_and_surface();
    for (alpha, field) in fields {
        for graph in [false, true] {
            let image = read(&mut renderer, &mut surface, field.clone().into(), graph);
            // H=1/4 gives k=ln(.5)/ln(.25)=1/2. At integer x=4,
            // P=1/4 => w=1/2; at x=9, P=9/16 => w=3/4.
            // With alpha .5 red, the premultiplied results are (.25,0,.5,.75)
            // and (.125,0,.75,.875), hence straight ratios 1:2 and 1:6.
            let (half, three_quarters) = if alpha == 1.0 {
                ([128, 0, 128, 255], [64, 0, 191, 255])
            } else {
                ([85, 0, 170, 191], [36, 0, 219, 223])
            };
            assert_pixel(&image, 4, 4, half, graph);
            assert_pixel(&image, 9, 4, three_quarters, graph);
        }
    }
}

#[test]
fn authentic_unequal_ellipse_hints_execute_for_ordinary_and_repeating_consumers() {
    let (ordinary, repeating) = {
        let geometry = paint::resolve_radial_gradient(
            layout::Size::new(32.0, 16.0),
            Some(layout::Point::new(8.0, 8.0)),
            paint::RadialGradientSize::EllipseRadii(layout::Size::new(
                position(16.0),
                position(8.0),
            )),
            options(),
        )
        .result
        .expect("authentic Paint ellipse preparation must succeed");
        let source = source(16.0, 1.0);
        let used = used(16.0, &source);
        let mut context = RenderAccountingContext::default();
        let ordinary = prepare_radial_gradient(&geometry, &used, &mut context)
            .expect("ordinary radial consumer must accept the authentic transition hint");
        let repeating = prepare_repeating_radial_gradient(&geometry, &used, &mut context)
            .expect("repeating radial consumer must accept the authentic transition hint");
        (ordinary, repeating)
    };
    let (mut renderer, mut surface) = renderer_and_surface();
    for graph in [false, true] {
        for (field, exterior) in [
            (Paint::from(ordinary.clone()), [0, 0, 255, 255]),
            (Paint::from(repeating.clone()), [128, 0, 128, 255]),
        ] {
            let image = read(&mut renderer, &mut surface, field, graph);
            // q=sqrt(((x-8)/16)^2+((y-8)/8)^2). The two quarter
            // rays share w=.5; q=9/16 gives w=.75. These are smooth
            // interior colors, with no assertion of an exact radial step tie.
            assert_pixel(&image, 12, 8, [128, 0, 128, 255], graph);
            assert_pixel(&image, 8, 10, [128, 0, 128, 255], graph);
            assert_pixel(&image, 17, 8, [64, 0, 191, 255], graph);
            // q=1.25 is Pad blue for ordinary, but repeats to P=.25.
            assert_pixel(&image, 28, 8, exterior, graph);
        }
    }
}

#[test]
fn repeating_hints_use_power_colors_when_resolved_and_stop_only_average_when_unresolved() {
    let fields = [
        (1.0, repeating_line(16.0, 1.0), repeating_line(1.0, 1.0)),
        (0.5, repeating_line(16.0, 0.5), repeating_line(1.0, 0.5)),
    ];
    let (mut renderer, mut surface) = renderer_and_surface();
    for (alpha, resolved, averaged) in fields {
        for graph in [false, true] {
            let image = read(&mut renderer, &mut surface, resolved.clone().into(), graph);
            let (half, three_quarters) = if alpha == 1.0 {
                ([128, 0, 128, 255], [64, 0, 191, 255])
            } else {
                ([85, 0, 170, 191], [36, 0, 219, 223])
            };
            assert_pixel(&image, 4, 4, half, graph);
            assert_pixel(&image, 9, 4, three_quarters, graph);
            assert_pixel(&image, 20, 4, half, graph);
            let image = read(&mut renderer, &mut surface, averaged.clone().into(), graph);
            // A one-device-pixel period is unresolved. Its specified average
            // is the adjacent-color trapezoid, excluding hints: w=.5. The
            // spatial integral of sqrt(P), w=2/3, would give different bytes.
            assert_pixel(&image, 4, 4, half, graph);
            assert_pixel(&image, 24, 12, half, graph);
        }
    }
}
