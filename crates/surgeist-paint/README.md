# surgeist-paint

Paint resolves linear and radial gradient geometry, classifies radial zero
cases, resolves shared stop positions, and plans CSS stacking and non-table
primitive order from explicit native facts.
`plan_stacking` returns context membership and immediate child context order.
`plan_paint_order` adds validated box/text occurrences, line incidences and
primitive records to return ordered borrowed references inside nested
actual-context intervals. Both use the shared context algorithm and one
Ledger operation for work admission, storage custody and optional profiling.
`resolve_linear_gradient` uses that same accounting authority to turn a concrete
image size and physical direction into box-local line endpoints and length.
`resolve_radial_gradient` supplies positive ellipse axes or an explicit ordered
degenerate case from a concrete radial size request.
`resolve_zero_height_gradient_colors` resolves a native zero-height case into
borrowed color weights for a solid result. Its [basis policy](docs/reference.md#zero-height-gradient-colors)
keeps mixed length/percentage semantics explicit.
`resolve_border_image` partitions an already-sized image and its outset paint
area into source regions and destination rectangles. When the source first
needs sizing, use `resolve_border_image_area` to obtain the default sizing
extent and pass the resolved area to `resolve_border_image_in_area`.

Callers retain exact source identities, geometry and appearance associations.
The order is not a Render command stream or a complete transform/clip tree.
Computed Style composition, table painting and complete scene lowering remain
unfinished; table/inline-table input receives a typed unsupported diagnostic.

## Start

See [getting started](docs/getting-started.md) for context planning and the
[ordering explanation](docs/explanation.md) for the supported native input profile.
This root-only request orders one already-resolved canvas appearance record:

```rust
use surgeist_layout::SourceIndex;
use surgeist_paint::*;

let boxes = [BoxFacts::new(
    None, SourceIndex::ZERO, ChildOrder::Source, Participation::Ordinary,
    Position::Static, ZIndex::Auto, ContextEffect::None,
)];
let appearance = [BoxPaint::new(BoxPaintKind::Block, FloatParticipation::None)];
let primitives = [PaintPrimitive::canvas(CanvasPaintKind::BackgroundColor, 0)];
let request = PaintRequest::new(
    &boxes, &appearance, &[], &[], &[], &primitives, BoxId::new(0),
);
let outcome = plan_paint_order(request, PaintOptions::new(
    PaintLimits::unlimited(), ProfileMode::Summary { timing: false },
));
let order = outcome.result.unwrap();
let mut events = order.events();
assert!(matches!(events.next(), Some(PaintOrderEvent::BeginContext(_))));
match events.next().unwrap() {
    PaintOrderEvent::Draw(primitive) => {
        assert_eq!(primitive.id(), PrimitiveId::new(0));
        assert!(std::ptr::eq(primitive.record(), &primitives[0]));
    }
    _ => unreachable!(),
}
assert!(matches!(events.next(), Some(PaintOrderEvent::EndContext(_))));
assert!(events.next().is_none());
assert_eq!(outcome.report.unwrap().live(ResourceKind::ScratchBytes), 0);
```

The [Layout composition tests](tests/paint_order_layout.rs) retain real completed
and fragmented-root geometry. The [native inline test](tests/paint_order_native_inline.rs)
joins finalized Text output to Layout placement and exact Paint slots. These use
explicit native CSS paint facts and do not authenticate computed Style.

For a concrete gradient box, supply its size separately from its placement:

```rust
use surgeist_layout::{Point, Size};
use surgeist_paint::{
    LinearGradientDirection, PaintLimits, PaintOptions, ProfileMode,
    resolve_linear_gradient,
};

let outcome = resolve_linear_gradient(
    Size::new(200.0, 100.0), LinearGradientDirection::Top,
    PaintOptions::new(PaintLimits::unlimited(), ProfileMode::Off),
);
let line = outcome.result.unwrap();
assert_eq!(line.start(), Point::new(100.0, 100.0));
assert_eq!(line.end(), Point::new(100.0, 0.0));
assert_eq!(line.length(), 100.0);
```

For radial geometry, select a native size request and an optional already-used
physical center. This side-sized ellipse has a rightward ray length of 100:

```rust
use surgeist_layout::{Point, Size};
use surgeist_paint::*;

let geometry = resolve_radial_gradient(
    Size::new(200.0, 100.0), None,
    RadialGradientSize::EllipseExtent(RadialGradientExtent::ClosestSide),
    PaintOptions::new(PaintLimits::unlimited(), ProfileMode::Off),
).result.unwrap();
assert_eq!(geometry.center(), Point::new(100.0, 50.0));
assert_eq!(geometry.case(), RadialGradientCase::Ellipse {
    radii: Size::new(100.0, 50.0),
});
```

The [linear](docs/explanation.md#linear-gradient-geometry) and
[radial](docs/explanation.md#radial-gradient-geometry-and-cases) explanations
describe geometry and numerical limits. Resolve native stop positions
against that length with `resolve_gradient_stops`:

```rust
use surgeist_paint::*;

let source = [
    GradientStopItem::Stop { payload: "red", positions: GradientStopPositions::Omitted },
    GradientStopItem::Stop { payload: "green", positions: GradientStopPositions::Omitted },
    GradientStopItem::Hint { position: GradientPosition { length: 0.0, fraction: 0.2 } },
    GradientStopItem::Stop { payload: "blue", positions: GradientStopPositions::Omitted },
];
let outcome = resolve_gradient_stops(100.0, &source, PaintOptions::new(
    PaintLimits::unlimited(), ProfileMode::Summary { timing: false },
));
let stops = outcome.result.unwrap();
let positions: Vec<_> = stops.items().map(|item| match item {
    UsedGradientItem::Stop { position, .. } | UsedGradientItem::Hint { position, .. } => position,
}).collect();
assert_eq!(positions, [0.0, 10.0, 20.0, 100.0]);
```

The result borrows its exact native source and cannot outlive those records.
The [operation's API example](src/gradient_stops.rs) demonstrates that lifetime
constraint.

See [used-position resolution](docs/explanation.md#gradient-stop-positions) for
double positions, hints and source association. Authored numeric/color/position
adapters, zero-circle radial realization and complete gradient scene composition
remain unfinished. Zero-width fields use basis-zero stops and Render's
[native consumer](../surgeist-render/docs/reference.md#horizontal-distance-fields).
Zero-height solid colors use the
[dedicated color measure](docs/reference.md#zero-height-gradient-colors).

## Documentation

| Guide | Purpose |
| --- | --- |
| [Getting started](docs/getting-started.md) | Construct and inspect plans and retain producer associations. |
| [How-to](docs/how-to.md) | Set budgets, inspect failure reports and run focused checks. |
| [Reference](docs/reference.md) | Find types, dimensions and source boundaries. |
| [Explanation](docs/explanation.md) | Understand ordering, identity and storage custody. |

## License and attribution

Authored in this repository under the [MIT License](../../LICENSE).
The [repository notice](../../NOTICE.md) records dependency provenance.
