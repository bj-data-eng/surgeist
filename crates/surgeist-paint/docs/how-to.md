# How-to

## Admit bounded work

Start with the explicit `PaintLimits::unlimited()` policy, then select named
`with_work_limit` and `with_resource_limit` caps and pass the limits to
`PaintOptions::new`. Named getters expose the selected policy. Zero limits are
meaningful. `PaintReport` exposes phase/refusal/event values resolved from Ledger
at construction; callers do not depend on numeric scope identities or name parsing.
`plan_stacking`, `plan_paint_order`, `resolve_linear_gradient` and
`resolve_gradient_stops`, `resolve_radial_gradient` and
`resolve_zero_height_gradient_colors` each apply this policy to one complete operation.
A refused operation returns no result. Its optional report retains earlier
accepted work and the refusal; dropped scratch/result custody has live usage
zero. `PaintError::Input` identifies a typed paint-input failure, including
unsupported categories/topologies, separately from budget or allocation failures.
Correct invalid input before retrying, or raise selected limits to retry the
unchanged borrowed data after a budget refusal.
For geometry, `PaintError::Gradient` distinguishes invalid width/height/angle
from an unrepresentable positive line. Retry corrected geometry or raise an
explicit named work/result-byte limit after budget refusal.
For stop lists, `PaintError::GradientStops` identifies an invalid length, list
structure or numeric component, or an unrepresentable result, with source
association where applicable. Keep source storage alive while using the result.
Bound `GradientStopVisit` for actual list/record walks and `RetainedResultBytes`
for allocated capacity plus the fixed result owner; a zero limit is meaningful.
For radial geometry, `PaintError::RadialGradient` distinguishes invalid native
inputs from an unrepresentable required calculation. Inspect the resolved case
before composing stops or fields. An explicit already-used center and a default
center calculation are different input facts; a default half that loses a
positive coordinate receives a numerical refusal.
Root usage is inclusive of child phases; inspect child attribution without
adding it to the root totals.

Select Ledger `ProfileMode::Off`, untimed/timed Summary, or bounded Detailed
observation. A supplied monotonic clock is configured with `with_clock`.
Detailed event loss does not change totals or admission. Timing phases surround
the actual synchronous algorithm; no async or GPU timing is claimed.

Retain or borrow the owned `PaintReport`; it has no allocating `Clone` operation.
Its resolved observation storage is reserved fallibly during setup. Match
`AccountingFailure::LedgerDefect` to inspect the original named Ledger defect;
Paint's own report-construction inconsistencies use `AccountingFailure::Invariant`.

## Check the crate

From the repository root, use the process policy in [AGENTS.md](../../../AGENTS.md):

```sh
cargo test --offline --locked -j 1 -p surgeist-paint
cargo clippy --offline --locked -j 1 -p surgeist-paint --all-targets --no-deps -- -F unsafe-code -D warnings
cargo fmt -p surgeist-paint -- --check
```

The [Layout order](../tests/paint_order_layout.rs) and
[native inline](../tests/paint_order_native_inline.rs) composition tests can also
be selected individually with `--test paint_order_layout` or
`--test paint_order_native_inline`. The crate test command includes the
[README example](../README.md#start) as a doctest.

[API audits](../../../docs/how-to.md#check-public-api-audits) use the root generator.
