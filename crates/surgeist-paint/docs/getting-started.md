# Getting started

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
