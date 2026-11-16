# surgeist-css-fuzz Package Guide

Read the [repository](../../../AGENTS.md), [CSS](../AGENTS.md) and
[shared fuzz](../../surgeist-fuzz/AGENTS.md) guides.

This verification package owns CSS fuzz cases, strict codecs, generators,
independent expectations, seeds and imported-corpus records.
Use only public `surgeist-css` and `surgeist-css-parser` front doors: no
private parser copy, no fuzz-only production export, and no production
dependency on this package or on `surgeist-fuzz`. The framework owns engines,
worker supervision, artifacts and same-failure reduction.

Read [Cargo.toml](Cargo.toml), [README.md](README.md) and [fuzz.toml](fuzz.toml)
for the targets, their revisioned bounds and the explicit exclusions.
Expectations come from the crate reference, the selected specifications and
CSS-owned metadata, never from a parser, serializer or earlier run; each
property names its authority in the source. A counterexample does not
authorize a production change: retain the exact minimized case and failure key
and return it to the coordinator. Never weaken a property, drop a generator
lane or widen a tolerance to make a run pass.

Seeds under `seeds/` are generated: rerun
`examples/materialize_seeds.rs` (source-qualified literals) or
`examples/import_csstree.rs` (CSSTree corpus inputs) and inspect the diff;
never hand-edit them or `corpus-import.toml`.

## Command Inventory

Run focused tests in default, property, coverage and all-feature modes, strict
all-target all-feature Clippy and formatting through bounded `pisct run` with
offline/locked Cargo and one build job. Campaigns use a clean committed source
checkout and a separate work directory; never against dirty source, and never
with tool or dependency acquisition.
