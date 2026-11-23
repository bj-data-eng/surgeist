#![cfg(all(feature = "render-paint", not(target_arch = "wasm32")))]
#![forbid(unsafe_code)]

//! Renders the whole border-image source at its concrete size before slicing.

use std::{
    convert::Infallible,
    future::Future,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
    thread,
    time::{Duration, Instant},
};

use surgeist::{css, layout, ledger::ProfileMode, paint, render};

use css::color::{
    CssAuthoredColorOrdinaryLimits, CssAuthoredColorOutcome, CssColorFactIdentity, CssColorFacts,
    CssColorPropertyFacts, CssColorSchemeFact, CssDeviceColorFact,
};
use css::property::CssKnownPropertyValueRef;
use css::values::image::{
    CssColorStopList, CssColorStopListItem, CssGradient, CssHorizontalGradientSide, CssImageValue,
    CssLinearGradientDirection, CssRadialExtent, CssRadialShape, CssRadialSize,
};
use paint::accounting::{PaintLimits, PaintOptions};
use paint::border_image::{BorderImageAreaRequest, BorderImageOutset, resolve_border_image_area};
use paint::gradient_stops::{
    GradientStopItem, GradientStopPositions, GradientStops, UsedGradientItem,
    resolve_gradient_stops,
};
use paint::linear_gradient::{LinearGradientDirection, resolve_linear_gradient};
use paint::radial_gradient::{
    RadialGradientCase, RadialGradientExtent, RadialGradientSize, resolve_radial_gradient,
};
use render::accounting::RenderAccountingContext;
use render::paint::{Color, Gradient, prepare_sdr_srgb_color};
use render::paint_gradient::{prepare_linear_gradient, prepare_radial_gradient};

#[path = "support/native_viewport.rs"]
mod native_viewport;
use native_viewport::NativeViewport;

struct BorderBox(layout::node::LayoutInput);

impl layout::tree::Traverse for BorderBox {
    type Node = ();
    type Scalar = f32;
    type Children<'a> = std::iter::Empty<()>;

    fn children(&self, (): ()) -> Self::Children<'_> {
        std::iter::empty()
    }

    fn child_count(&self, (): ()) -> usize {
        0
    }

    fn child(&self, (): (), index: usize) {
        panic!("the standalone border box has no child at index {index}");
    }
}

impl layout::tree::LayoutTree for BorderBox {
    type MeasureError = Infallible;

    fn node_input(&self, (): ()) -> &layout::node::NodeInput {
        self.0.as_box().unwrap()
    }

    fn layout_input(&self, (): ()) -> layout::node::LayoutInputView<'_> {
        self.0.as_view()
    }
}

fn paint_options() -> PaintOptions {
    PaintOptions::new(PaintLimits::unlimited(), ProfileMode::Off)
}

fn source_size() -> layout::geometry::Size<f64> {
    let border =
        layout::length::LengthOf::value(layout::length::LengthPercentageOf::px(2.0).unwrap());
    let px = |length| {
        layout::sizing::PreferredSize::value(
            layout::length::LengthPercentageOf::px(length).unwrap(),
        )
    };
    let tree = BorderBox(layout::node::LayoutInput::box_input(
        layout::node::NodeInput {
            display: layout::box_model::Display::Block,
            box_sizing: layout::box_model::BoxSizing::BorderBox,
            size: layout::geometry::Size::new(px(28.0), px(12.0)),
            border: layout::geometry::Edges::all(border),
            ..layout::node::NodeInput::DEFAULT
        },
    ));
    let request = NativeViewport::new(
        layout::geometry::Size::splat(Some(64.0)),
        layout::geometry::FlowAxes::new(
            layout::geometry::WritingMode::HorizontalTb,
            layout::geometry::Direction::Ltr,
        ),
    )
    .request(layout::geometry::Size::splat(
        layout::constraint::SizingOperation::Normal,
    ));
    let batch = layout::compute_layout(&tree, (), request).unwrap();
    let node = batch.node_layout(()).unwrap().single_box().unwrap();
    let output = node.final_output();
    assert_eq!(output.size, layout::geometry::Size::new(28.0, 12.0));

    let area = resolve_border_image_area(
        BorderImageAreaRequest {
            border_origin: output.location.map(f64::from),
            border_size: output.size.map(f64::from),
            border_widths: output.border.map(f64::from),
            outsets: layout::geometry::Edges::all(BorderImageOutset::BorderWidthMultiple(1.0)),
        },
        paint_options(),
    )
    .result
    .unwrap();

    // CSS Images 3 §4.4 sizes the whole source using the border-image area as
    // its default object size. A gradient has no natural dimensions.
    let size = area.default_size();
    assert_eq!(size, layout::geometry::Size::new(32.0, 16.0));
    size
}

fn authored_source(declaration: &css::declaration::CssDeclaration) -> &CssGradient {
    let value = declaration.known().unwrap().property_value().unwrap();
    let CssKnownPropertyValueRef::BorderImageSource(source) = value else {
        panic!("expected a typed border-image-source declaration");
    };
    let CssImageValue::Gradient(gradient) = source.value() else {
        panic!("expected an authored gradient source");
    };
    gradient
}

fn omitted_stops(
    list: &CssColorStopList,
    account: &mut RenderAccountingContext,
) -> [GradientStopItem<Color>; 2] {
    let [
        CssColorStopListItem::Stop(first),
        CssColorStopListItem::Stop(last),
    ] = list.items()
    else {
        panic!("expected exactly two authored color stops");
    };
    // The intrinsic colors have no element, palette, scheme, or device dependencies.
    let facts = CssColorFacts::new(
        CssColorFactIdentity::new(1, 0),
        CssColorPropertyFacts::InitialProperties,
        CssColorSchemeFact::Unavailable,
        &[],
        &[],
        &[],
        CssDeviceColorFact::Absent,
    );
    [first, last].map(|stop| {
        assert!(stop.positions().is_empty());
        let resolved = stop
            .color()
            .resolve(
                &facts,
                facts.identity(),
                CssAuthoredColorOrdinaryLimits::default(),
            )
            .unwrap();
        let CssAuthoredColorOutcome::Used(color) = resolved.outcome() else {
            panic!("expected an intrinsic used color for the legacy sRGB stop");
        };
        GradientStopItem::Stop {
            payload: prepare_sdr_srgb_color(*color, account).unwrap(),
            positions: GradientStopPositions::Omitted,
        }
    })
}

fn used_stops(
    basis: f64,
    expected_end: f64,
    source: &[GradientStopItem<Color>],
) -> GradientStops<'_, Color> {
    let stops = resolve_gradient_stops(basis, source, paint_options())
        .result
        .unwrap();
    let observed: Vec<_> = stops
        .items()
        .map(|item| {
            let UsedGradientItem::Stop {
                source_index,
                position,
                ..
            } = item
            else {
                panic!("two authored colors should produce only used stops");
            };
            (source_index, position)
        })
        .collect();
    assert_eq!(observed, [(0, 0.0), (1, expected_end)]);
    stops
}

fn linear_source(
    source: &CssGradient,
    size: layout::geometry::Size<f64>,
    account: &mut RenderAccountingContext,
) -> Gradient {
    let CssGradient::Linear(linear) = source else {
        panic!("expected an ordinary linear gradient");
    };
    assert!(linear.interpolation().is_none());
    let direction = match linear.direction() {
        Some(CssLinearGradientDirection::SideOrCorner(side))
            if side.horizontal() == Some(CssHorizontalGradientSide::Right)
                && side.vertical().is_none() =>
        {
            LinearGradientDirection::Right
        }
        other => panic!("expected the authored direction to right, got {other:?}"),
    };
    let line = resolve_linear_gradient(size, direction, paint_options())
        .result
        .unwrap();
    let source = omitted_stops(linear.stops(), account);
    let stops = used_stops(line.length(), 32.0, &source);
    let gradient = prepare_linear_gradient(&line, &stops, account).unwrap();
    let (start, end) = gradient.linear_points().unwrap();
    assert_eq!(
        (start.x(), start.y(), end.x(), end.y()),
        (0.0, 8.0, 32.0, 8.0)
    );
    gradient
}

fn radial_source(
    source: &CssGradient,
    size: layout::geometry::Size<f64>,
    account: &mut RenderAccountingContext,
) -> Gradient {
    let CssGradient::Radial(radial) = source else {
        panic!("expected an ordinary radial gradient");
    };
    assert!(radial.interpolation().is_none());
    let (size_request, center) = match (radial.shape(), radial.size(), radial.position()) {
        (
            Some(CssRadialShape::Circle),
            Some(CssRadialSize::Extent(CssRadialExtent::ClosestSide)),
            None,
        ) => (
            RadialGradientSize::CircleExtent(RadialGradientExtent::ClosestSide),
            None,
        ),
        other => {
            panic!("expected the authored circle closest-side with omitted center, got {other:?}")
        }
    };
    let geometry = resolve_radial_gradient(size, center, size_request, paint_options())
        .result
        .unwrap();
    let RadialGradientCase::Ellipse { radii } = geometry.case() else {
        panic!("expected a positive-radius circle");
    };
    let source = omitted_stops(radial.stops(), account);
    let stops = used_stops(radii.width, 8.0, &source);
    let gradient = prepare_radial_gradient(&geometry, &stops, account).unwrap();
    let (center, radius_x, radius_y) = gradient.radial_geometry().unwrap();
    assert_eq!(
        (center.x(), center.y(), radius_x, radius_y),
        (16.0, 8.0, 8.0, 8.0)
    );
    gradient
}

#[test]
fn authored_border_image_gradients_render_at_the_concrete_source_size() {
    // Legacy rgba() and named colors select sRGB under CSS Color 4 §13.2.
    let parser = css::CssParser::default();
    let [linear_report, radial_report] = [
        "border-image-source: linear-gradient(to right, rgba(255, 0, 0, 0.5), blue);",
        "border-image-source: radial-gradient(circle closest-side, rgba(255, 0, 0, 0.5), blue);",
    ]
    .map(|source| {
        let report = parser.parse_block_contents(source);
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        assert_eq!(report.syntax().len(), 1);
        report
    });
    let size = source_size();
    let mut account = RenderAccountingContext::default();
    let linear = linear_source(
        authored_source(&linear_report.syntax()[0]),
        size,
        &mut account,
    );
    let radial = radial_source(
        authored_source(&radial_report.syntax()[0]),
        size,
        &mut account,
    );
    let expected_stops = [
        (0.0, Color::try_rgba(1.0, 0.0, 0.0, 0.5).unwrap()),
        (1.0, Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap()),
    ];
    for gradient in [&linear, &radial] {
        let stops: Vec<_> = gradient
            .stops()
            .iter()
            .map(|stop| (stop.position(), stop.color()))
            .collect();
        assert_eq!(stops, expected_stops);
    }

    run_headless(async {
        let mut renderer = render::renderer::Renderer::new(render::renderer::Options::default())
            .await
            .unwrap();
        let mut surface = renderer
            .create_headless(
                render::geometry::Size::try_new(size.width, size.height).unwrap(),
                1.0,
            )
            .await
            .unwrap();
        let source = render::geometry::Rect::try_new(0.0, 0.0, size.width, size.height).unwrap();

        // At t=.5, premultiplied sRGB is (.25, 0, .5, .75), so straight
        // RGBA8 readback is approximately (85, 0, 170, 191).
        let midpoint = [85, 0, 170, 191];
        for (gradient, samples) in [
            (
                linear,
                [
                    (8, 8, [153, 0, 102, 159]),
                    (16, 8, midpoint),
                    (24, 8, [36, 0, 219, 223]),
                ],
            ),
            (
                radial,
                [
                    (20, 8, midpoint),
                    (16, 12, midpoint),
                    (26, 8, [0, 0, 255, 255]),
                ],
            ),
        ] {
            let mut scene = render::scene::Scene::new();
            scene.fill(source, gradient);
            renderer
                .render(&mut surface, &scene, render::surface::Parameters::default())
                .await
                .unwrap();
            let buffer = renderer.read_headless(&surface).await.unwrap();
            for (x, y, expected) in samples {
                let offset = usize::try_from((y * buffer.size().width() + x) * 4).unwrap();
                let pixel = &buffer.rgba()[offset..offset + 4];
                // Allow for ramp sampling and RGBA8 premultiply/unpremultiply rounding.
                assert!(
                    pixel
                        .iter()
                        .zip(expected)
                        .all(|(&actual, expected)| actual.abs_diff(expected) <= 3),
                    "pixel ({x}, {y}): got {pixel:?}, expected {expected:?}"
                );
            }
        }
    });
}

fn run_headless(future: impl Future<Output = ()>) {
    struct WakeThread(thread::Thread);
    impl Wake for WakeThread {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
    }

    let waker = Waker::from(Arc::new(WakeThread(thread::current())));
    let mut context = Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        if let Poll::Ready(()) = future.as_mut().poll(&mut context) {
            return;
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        assert!(
            !remaining.is_zero(),
            "native gradient render exceeded 60 seconds"
        );
        thread::park_timeout(remaining);
    }
}
