# How-to

## Admit bounded work

Start with the explicit `PaintLimits::unlimited()` policy, then select named
`with_work_limit` and `with_resource_limit` caps and pass the limits to
`PaintOptions::new`. Named getters expose the selected policy. Zero limits are
meaningful. `PaintReport` exposes named phase/refusal/event views backed by
Ledger; callers do not depend on an internal numeric schema.
Both `plan_stacking` and `plan_paint_order` apply this policy to one complete
operation. A refused operation returns no plan. Its optional report retains
earlier accepted work and the refusal; dropped scratch/result custody has live
usage zero. `PaintError::Input` identifies a typed paint-input failure, including
unsupported categories/topologies, separately from budget or allocation failures.
Correct invalid input before retrying, or raise selected limits to retry the
unchanged borrowed data after a budget refusal.
Root usage is inclusive of child phases; inspect child attribution without
adding it to the root totals.

Select Ledger `ProfileMode::Off`, untimed/timed Summary, or bounded Detailed
observation. A supplied monotonic clock is configured with `with_clock`.
Detailed event loss does not change totals or admission. Timing phases surround
the actual synchronous algorithm; no async or GPU timing is claimed.

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
