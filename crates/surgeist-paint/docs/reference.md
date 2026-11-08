# Reference

The [manifest](../Cargo.toml) owns dependencies and Rust compatibility.
The [public front door](../src/lib.rs) exports these surfaces:

| Surface | Contract |
| --- | --- |
| `BoxId`, `BoxFacts` | Rendering-box slots and explicit resolved topology/context facts. |
| `ChildOrder`, `Participation`, `Position`, `ZIndex`, `ContextEffect` | Shared source/flex/grid traversal and actual-context classification facts. |
| `plan_stacking`, `PaintOutcome`, `StackingPlan`, `ContextView` | Context membership and ordered immediate child contexts, or typed failure. |
| `BoxPaint`, `BoxPaintKind`, `FloatParticipation` | Same-slot resolved box categories and float participation. |
| `TextSource`, `PaintLine`, `PaintFragment`, `BlockInInlinePlacement` | Logical sources and exact line/fragment occurrences with explicit promotion placement. |
| `TextId`, `LineId`, `FragmentId`, `PrimitiveId` | Distinct request-local table slots, without global provenance. |
| `PaintPrimitive`, `PrimitiveKind`, `CanvasPaintKind`, `FragmentPaintKind`, `DecorationKind` | Resolved appearance roles and local record order. |
| `PaintRequest`, `plan_paint_order`, `PaintOrderOutcome`, `PaintOrder` | Borrowed resolved input and complete supported order, or typed failure. |
| `PaintOrderEvent`, `BoxView`, `LineView`, `FragmentView`, `PrimitiveView` | Actual-context boundaries and exact borrowed source-record observations. |
| `PaintError`, `PaintInputError`, `BoxCategoryError` | Planning/accounting failures and typed input/category/unsupported-profile diagnostics. |
| `PaintLimits`, `PaintOptions`, `PaintReport`, `PaintPhase` | Limits, clock/profiling policy and named Ledger usage access. |
| `resolve_linear_gradient`, `LinearGradientDirection`, `LinearGradientLine`, `LinearGradientOutcome` | Concrete box-local physical line geometry, length and optional accepted-prefix report. |
| `GradientGeometryError` | Invalid dimensions/angle or unrepresentable positive line, wrapped by `PaintError::Gradient`. |
| `GradientPosition`, `GradientStopPositions`, `GradientStopItem<C>` | Raw affine positions and native source records with opaque stop payloads or distinct hints. |
| `resolve_gradient_stops`, `GradientStopsOutcome`, `GradientStops`, `UsedGradientItem` | Complete ordered used positions with exact source association, or typed failure and optional report. |
| `GradientStopsError`, `GradientPositionComponent` | List/length/component/numerical diagnostics with source indices and expanded-position ordinals, wrapped by `PaintError::GradientStops`. |

## Accounting schema

The operations share 20 work dimensions and two byte dimensions. Actions count
algorithm work, not CPU instructions or elapsed time.

| Work family | `WorkKind` dimensions |
| --- | --- |
| Shared stacking and output | `BoxValidation`, `ParentEdge`, `TopologyVisit`, `SiblingComparison`, `MembershipVisit`, `ContextComparison`, `OrderMove`, `OutputRecord` |
| Paint input and sequencing | `PaintValidation`, `FragmentVisit`, `LineVisit`, `InlineMemberVisit`, `PrimitiveVisit`, `PaintScopeVisit`, `PaintOrderComparison`, `DecorationComparison` |
| Gradient geometry | `GradientValidation`, `GradientDirection`, `GradientCoordinate` |
| Gradient stop lists | `GradientStopVisit`, plus shared validation, coordinate and output dimensions |

`ResourceKind` is `ScratchBytes` or `RetainedResultBytes`.
Fixed child phases are `validate`, `topology`, `sibling_order`, `membership`,
`context_order`, `publication` and `cleanup`. `plan_stacking` has these seven
children under the `paint stacking` root: eight scopes in total.
`plan_paint_order` adds `paint_validate` and `paint_sequence` under the
`paint order` root: nine children and ten scopes in total. It reuses stacking
inside that single operation/report. Phase usage is inclusive; root totals and
child attribution must not be summed together.

`resolve_linear_gradient` uses gradient validation, geometry and publication
attribution. Validation counts each input scalar or direction discriminant;
direction work counts actual resolution actions. Coordinate work counts actual
scalar geometry calculations and inspections, including repeated work. A successful
operation admits one output record and the fixed `LinearGradientLine` result's
size as retained bytes. It allocates no geometry buffer; caller input and
Ledger's bounded observer remain outside these byte dimensions. See
[the implementation](../src/gradient.rs) for the precise charged action units.

`resolve_gradient_stops` uses the `paint gradient stops` root with six child
phases: validation, stop resolution, prefix fixup, omitted distribution,
publication and cleanup. `GradientStopVisit` counts each actual source or
expanded-record visit, including repeated passes and boundary inspection.
`GradientCoordinate` counts actual numeric calculations/conversions/inspections;
`GradientValidation` counts input domain/grammar actions. Each expanded retained
record and the final fixed result owner contribute an `OutputRecord`.
Retained bytes include actual record capacity and the fixed owner; source and
payload storage remain caller-owned. See
[the implementation](../src/gradient_stops.rs) for the precise charged units.

## Source map

[Accounting](../src/accounting.rs) owns the named schema and capacity admissions.
[The context algorithm](../src/algorithm.rs) owns shared classification,
traversal and fallible sorting; [stacking models](../src/model.rs) own context
views. [Paint-order models](../src/paint_order/model.rs) own request records,
typed input errors and borrowed order views. [Validation](../src/paint_order/validation.rs)
owns cross-table incidence; [sequencing](../src/paint_order/sequencing.rs) owns
paint bands and recursive scope ordering. The [operation](../src/paint_order/mod.rs)
composes these under one report.
[Gradient geometry](../src/gradient.rs) owns physical direction resolution,
numerical validation and the line result; its
[public tests](../tests/linear_gradient.rs) use real Layout values and independent
coordinate, budget and profiling expectations.
[Shared stop resolution](../src/gradient_stops.rs) owns expansion, affine
positions, ordered fixup, omitted runs and borrowed result views;
[public stop tests](../tests/gradient_stops.rs) exercise its values, associations
and accounting through the front door.

[Stacking tests](../src/algorithm/tests.rs), [paint-order tests](../src/paint_order/tests.rs)
and [accounting tests](../src/paint_order/accounting_tests.rs) supply literal order,
input-refusal and work/storage-boundary expectations. Public producer composition
is covered by [Layout](../tests/paint_order_layout.rs) and
[native inline](../tests/paint_order_native_inline.rs) tests.
