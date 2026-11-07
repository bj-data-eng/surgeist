# Reference

The [manifest](../Cargo.toml) owns dependencies and Rust compatibility.
The [public front door](../src/lib.rs) exports:

| Surface | Contract |
| --- | --- |
| `BoxId`, `BoxFacts` | Borrowed rendering-box slots and explicit resolved facts. |
| `ChildOrder`, `Participation` | Source versus flex/grid child order and item participation. |
| `Position`, `ZIndex`, `ContextEffect` | CSS categories, exact resolved i64 level and applicable effect trigger. |
| `plan_stacking`, `PaintOutcome`, `PaintError` | Complete plan or typed failure, plus optional accepted-prefix report. |
| `StackingPlan`, `ContextView` | Per-box actual context and borrowed ordered child-context traversal. |
| `PaintLimits`, `PaintOptions`, `PaintReport` | Limits, clock/profiling policy and named Ledger usage access. |

Work dimensions are BoxValidation, ParentEdge, TopologyVisit, SiblingComparison,
MembershipVisit, ContextComparison, OrderMove and OutputRecord.
Byte dimensions are ScratchBytes and RetainedResultBytes. Fixed named phases
are validate, topology, sibling_order, membership, context_order, publication
and cleanup, under the paint stacking root.

[Accounting](../src/accounting.rs) owns the schema and actual-capacity admissions;
[algorithm](../src/algorithm.rs) owns action charging and fallible merge sorting;
[models](../src/model.rs) own the resolved values and plan views.
The [unit tests](../src/algorithm/tests.rs) trace literal work and storage limits.
