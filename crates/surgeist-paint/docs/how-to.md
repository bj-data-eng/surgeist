# How-to

## Admit bounded work

Start with the explicit `PaintLimits::unlimited()` policy, then select named
`with_work_limit` and `with_resource_limit` caps and pass the limits to
`PaintOptions::new`. Named getters expose the selected policy. Zero limits are
meaningful. `PaintReport` exposes named phase/refusal/event views backed by
Ledger; callers do not depend on an internal numeric schema.
A refused operation returns no plan. Its optional report retains earlier
accepted work and the refusal; dropped scratch/result custody has live usage zero.

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

[API audits](../../../docs/how-to.md#check-public-api-audits) use the root generator.
