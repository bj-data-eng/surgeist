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

## Accounting schema

Both operations share 16 work dimensions and two byte dimensions. Actions count
algorithm work, not CPU instructions or elapsed time.

| Work family | `WorkKind` dimensions |
| --- | --- |
| Shared stacking and output | `BoxValidation`, `ParentEdge`, `TopologyVisit`, `SiblingComparison`, `MembershipVisit`, `ContextComparison`, `OrderMove`, `OutputRecord` |
| Paint input and sequencing | `PaintValidation`, `FragmentVisit`, `LineVisit`, `InlineMemberVisit`, `PrimitiveVisit`, `PaintScopeVisit`, `PaintOrderComparison`, `DecorationComparison` |

`ResourceKind` is `ScratchBytes` or `RetainedResultBytes`.
Fixed child phases are `validate`, `topology`, `sibling_order`, `membership`,
`context_order`, `publication` and `cleanup`. `plan_stacking` has these seven
children under the `paint stacking` root: eight scopes in total.
`plan_paint_order` adds `paint_validate` and `paint_sequence` under the
`paint order` root: nine children and ten scopes in total. It reuses stacking
inside that single operation/report. Phase usage is inclusive; root totals and
child attribution must not be summed together.

## Source map

[Accounting](../src/accounting.rs) owns the named schema and capacity admissions.
[The context algorithm](../src/algorithm.rs) owns shared classification,
traversal and fallible sorting; [stacking models](../src/model.rs) own context
views. [Paint-order models](../src/paint_order/model.rs) own request records,
typed input errors and borrowed order views. [Validation](../src/paint_order/validation.rs)
owns cross-table incidence; [sequencing](../src/paint_order/sequencing.rs) owns
paint bands and recursive scope ordering. The [operation](../src/paint_order/mod.rs)
composes these under one report.

[Stacking tests](../src/algorithm/tests.rs), [paint-order tests](../src/paint_order/tests.rs)
and [accounting tests](../src/paint_order/accounting_tests.rs) supply literal order,
input-refusal and work/storage-boundary expectations. Public producer composition
is covered by [Layout](../tests/paint_order_layout.rs) and
[native inline](../tests/paint_order_native_inline.rs) tests.
