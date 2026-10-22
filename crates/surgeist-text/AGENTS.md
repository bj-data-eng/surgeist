# surgeist-text Crate Guide

Read the [repository guide](../../AGENTS.md) for shared policy and workflow.
This supplement records crate facts and grants no additional authority.

## Product Boundary

This crate owns text composition/edits, font-facing inputs, shaping, measurement, line
layout, geometry, cursor/selection movement, and cache identity. Source IDs do
not make this crate the document owner. Style resolution, rendering engines,
widgets, and application commands remain outside this crate. The existing
`text-render` dependency and `Layout::push_render_text` bridge remain a known
boundary discrepancy; the snapshot import does not reconcile it.
`text-accessibility` exposes AccessKit projection.

## Discover The Current Structure

| Fact | Authoritative source |
| --- | --- |
| Package identity, dependencies, features and targets | [Cargo.toml](Cargo.toml) |
| Public front door | [src/lib.rs](src/lib.rs) and its reexports |
| Source composition, ranges, validation, and edits | [src/source.rs](src/source.rs), [src/composer.rs](src/composer.rs), [src/range.rs](src/range.rs), [src/edit.rs](src/edit.rs) |
| Text style inputs, supported features, options, and diagnostics | [src/style.rs](src/style.rs), [src/style_support.rs](src/style_support.rs), [src/options.rs](src/options.rs), [src/error.rs](src/error.rs) |
| Layout construction and cache identity | [src/system.rs](src/system.rs), [src/catalogue.rs](src/catalogue.rs), [src/cache.rs](src/cache.rs) |
| Layout outputs, geometry, movement, and optional projections | [src/layout.rs](src/layout.rs), [src/geometry.rs](src/geometry.rs) |
| Focused verification | [src/tests.rs](src/tests.rs), its [catalogue cases](src/tests/catalogue.rs), and owning module tests, loaded by [src/lib.rs](src/lib.rs) |

[README.md](README.md) links usage, contract and attribution documentation.

## Command Inventory

Select affected checks under the root process/resource policy; commands below
run from this crate directory.

The deterministic registered-font fixture is [Ahem](tests/fixtures/fonts/ahem/PROVENANCE.md).

Default features are empty. Optional surfaces are `text-accessibility` and
`text-render`. The current text-render bridge has pre-existing API mismatches
against `surgeist-render`; do not assume it or an all-features combination passes.
Record that limitation separately from default/accessibility checks. Render
feature tests initialize a headless renderer and require a usable GPU backend.

```sh
cargo check -j 1 -p surgeist-text
cargo test -j 1 -p surgeist-text
cargo clippy -j 1 -p surgeist-text --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-text -- --check
```
