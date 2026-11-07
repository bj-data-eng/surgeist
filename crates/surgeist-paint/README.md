# surgeist-paint

Paint plans resolved CSS stacking and non-table primitive order from explicit
native facts. `plan_stacking` returns context membership and immediate child
context order. `plan_paint_order` adds validated box/text occurrences, line
incidences and primitive records to return ordered borrowed references inside
nested actual-context intervals. Both use the shared context algorithm and one
Ledger operation for work admission, storage custody and optional profiling.

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
