# surgeist-text Crate Guide

Read the [repository guide](../../AGENTS.md) for shared policy and workflow.
This supplement records crate facts and grants no additional authority.

## Product Boundary

This crate owns text composition/edits, font-facing inputs, shaping, measurement, line
layout, geometry, cursor/selection movement, and cache identity. Source IDs do
not make this crate the document owner. Style resolution, rendering engines,
widgets, and application commands remain outside this crate. Render owns the
optional standalone paint adapter and consumes Text directly.
`text-accessibility` exposes AccessKit projection.

## Discover The Current Structure

| Fact | Authoritative source |
| --- | --- |
| Package identity, dependencies, features and targets | [Cargo.toml](Cargo.toml) |
| Public front door | [src/lib.rs](src/lib.rs) and its reexports |
| Source composition, ranges, validation, and edits | [src/source.rs](src/source.rs), [src/composer.rs](src/composer.rs), [src/range.rs](src/range.rs), [src/edit.rs](src/edit.rs) |
| Exact UTF-16 storage and scalar provider mapping | [src/text.rs](src/text.rs), [src/projection.rs](src/projection.rs) |
| Pre-font whitespace processing, correspondence and partial plaintext | [src/processing.rs](src/processing.rs), [owning tests](src/processing/tests.rs) |
| Text style inputs, supported features, options, and diagnostics | [src/style.rs](src/style.rs), [src/style_support.rs](src/style_support.rs), [src/options.rs](src/options.rs), [src/error.rs](src/error.rs) |
| Layout construction and cache identity | [src/system.rs](src/system.rs), [src/catalogue.rs](src/catalogue.rs), [src/cache.rs](src/cache.rs) |
| Canonical font resources/instances and glyph-free static metrics | [src/font.rs](src/font.rs), [metrics](src/font/metrics.rs) |
| Layout outputs, geometry, movement, and optional projections | [src/layout.rs](src/layout.rs), [src/geometry.rs](src/geometry.rs) |
| Borrowed actual run, face and instance readback | [src/views.rs](src/views.rs) |
| Owned optional AccessKit projection | [src/accessibility.rs](src/accessibility.rs), [provenance](docs/accessibility-provenance.md) |
| Focused verification | [src/tests.rs](src/tests.rs), its [catalogue cases](src/tests/catalogue.rs), and owning module tests, loaded by [src/lib.rs](src/lib.rs) |

[README.md](README.md) links usage, contract and attribution documentation.

## Command Inventory

Select affected checks under the root process/resource policy; commands below
run from this crate directory.

The deterministic registered-font fixture is [Ahem](tests/fixtures/fonts/ahem/PROVENANCE.md).

Default features are empty. `text-accessibility` enables owned AccessKit
projection. Render adapter tests belong to the consuming `surgeist-render`
package under its `render-text` feature.

```sh
cargo check -j 1 -p surgeist-text
cargo test -j 1 -p surgeist-text
cargo clippy -j 1 -p surgeist-text --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-text -- --check
```
