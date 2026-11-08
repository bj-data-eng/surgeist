# Getting started

## Resolve a native gradient

Use `resolve_linear_gradient` with the concrete Layout size and physical
direction, then pass its returned length to `resolve_gradient_stops`. Keep the
native source list alive: used stop views borrow its exact payloads and preserve
source indices, including both occurrences of a double-position stop. The
[crate examples](../README.md#start) show geometry and a hint-bounded omitted
position; the [explanation](explanation.md#gradient-stop-positions) describes the
remaining authored-value and rendering boundaries.

For radial geometry, supply a coupled `RadialGradientSize` and either an
already-used Layout point or `None` for the box center. Match the returned case.
`RadialGradientCase::Ellipse` provides positive axes; its horizontal radius is
the ordinary rightward ray length for `resolve_gradient_stops`. Handle the
three degenerate cases explicitly rather than treating them as zero ordinary
radii. For `ZeroHeight`, use
[`resolve_zero_height_gradient_colors`](reference.md#zero-height-gradient-colors)
with the raw source and ordinary/repeating selection to obtain a solid color
measure. For `ZeroWidth`, resolve raw stops with basis zero and pass geometry
and used stops to Render's ordinary or repeating radial consumer. Its
[horizontal-distance field](../../surgeist-render/docs/reference.md#horizontal-distance-fields)
mirrors physical positions around the center's x coordinate. Zero-circle
realization remains separate composition work.

## Plan stacking contexts

A box ID is a slot in one immutable fact slice. Supply parentage, source sibling
indices and resolved CSS facts; input slice order is not painting order.

```rust
use surgeist_layout::SourceIndex;
use surgeist_paint::*;

let boxes = [
    BoxFacts::new(None, SourceIndex::ZERO, ChildOrder::Source,
        Participation::Ordinary, Position::Static, ZIndex::Auto, ContextEffect::None),
    BoxFacts::new(Some(BoxId::new(0)), SourceIndex::ZERO, ChildOrder::Source,
        Participation::Ordinary, Position::Relative, ZIndex::Integer(-1), ContextEffect::None),
];
let options = PaintOptions::new(PaintLimits::unlimited()
    .with_work_limit(WorkKind::BoxValidation, 100)
    .with_resource_limit(ResourceKind::ScratchBytes, 10_000)
    .with_resource_limit(ResourceKind::RetainedResultBytes, 10_000),
    ProfileMode::Summary { timing: false });
let outcome = plan_stacking(&boxes, BoxId::new(0), options);
let plan = outcome.result.unwrap();
assert_eq!(plan.root_context().children().next().unwrap().box_id(), BoxId::new(1));
assert_eq!(plan.context_for_box(BoxId::new(1)).unwrap().stack_level(), -1);
assert_eq!(outcome.report.unwrap().live(ResourceKind::ScratchBytes), 0);
```

Keep actual Layout node identities and completed geometry in a parallel
caller-owned collection keyed by box slot. See the
[real Layout composition test](../tests/layout_composition.rs).

## Plan primitive order

Use `plan_stacking` when context membership and child order suffice. To place
primitive appearances among them, use `plan_paint_order` with a `PaintRequest`;
the [crate example](../README.md#start) shows a minimal canvas request.

Add one `BoxPaint` for each `BoxFacts` slot. Add logical text sources and exact
line/fragment occurrences independently of geometry availability. Add primitive
records for resolved canvas, background, border, replaced content, text,
decoration and outline appearances as applicable. These records specify roles
and local appearance order, while Paint chooses the CSS phase. Root background
records target the canvas, not each root fragment.

After a successful operation, iterate `PaintOrder::events()`. `Draw` supplies a
borrowed `PrimitiveView` with its exact request record and slot; use that slot to
look up the caller's retained appearance/geometry association. Actual context
boundaries delimit atomic ordering intervals. Typed input or budget failure
publishes no partial order; inspect the optional report before retrying.

The [Layout order test](../tests/paint_order_layout.rs) maps completed fragments,
including fragmented roots, to request slots. The [native inline test](../tests/paint_order_native_inline.rs)
keeps finalized Text fragments and Layout line/run associations distinct even
when producer-local line indices repeat. Both supply native resolved CSS facts
explicitly. See [the explanation](explanation.md#occurrences-and-primitive-order)
for supported topology and the limits of these associations.
