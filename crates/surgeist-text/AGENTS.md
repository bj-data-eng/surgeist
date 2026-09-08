# surgeist-text Local Guide

This crate is part of the Surgeist repository and product Cargo workspace.
Read [../../AGENTS.md](../../AGENTS.md) first for workflow, architecture, shared
verification/resource limits, and API generation. This supplement records local
facts; it grants no additional authority. The root guide prohibits executable
`unsafe` in all Surgeist-owned Rust.

This crate owns text composition/edits, font-facing inputs, shaping, measurement, line
layout, geometry, cursor/selection movement, and cache identity. Source IDs do
not make this crate the document owner. Style resolution, rendering engines,
widgets, and application commands remain outside this crate. The existing
`text-render` dependency and `Layout::push_render_text` bridge remain a known
boundary discrepancy; the snapshot import does not reconcile it.
`text-accessibility` exposes AccessKit projection.

Consume sibling public front doors. Cross-crate lowering and composition remain
at the root boundary defined by the root guide.

## Sources

[Cargo.toml](Cargo.toml) owns local package/feature facts;
[../../Cargo.toml](../../Cargo.toml) and [../../Cargo.lock](../../Cargo.lock)
own product workspace membership and dependency resolution. Public source remains
authoritative; root-owned API audits are generated through
[../../api/generator](../../api/generator), never hand-edited or copied here.

| Fact | Source |
| --- | --- |
| Package identity, version, edition, dependencies, features, and targets | `Cargo.toml` |
| Public front door | `src/lib.rs` and its reexports |
| Human entry point and documentation navigation | `README.md` |
| Source composition, ranges, validation, and edits | `src/source.rs`, `src/composer.rs`, `src/range.rs`, `src/edit.rs` |
| Text style inputs, supported features, options, and diagnostics | `src/style.rs`, `src/style_support.rs`, `src/options.rs`, `src/error.rs` |
| Layout construction and cache identity | `src/system.rs`, `src/cache.rs` |
| Layout outputs, geometry, movement, and optional projections | `src/layout.rs`, `src/geometry.rs` |
| Focused verification | `src/tests.rs`, loaded by `src/lib.rs` |
| Configured commands and feature gates | `Cargo.toml`, this command inventory, and tracked task-runner or CI configuration when present |
| Project license | `LICENSE` |
| Direct-dependency attribution scope and included legal material | `NOTICE.md`, `licenses/`, and the exact dependencies in `Cargo.toml` |

## Local Verification

Run the applicable commands from this crate directory, one package at a time,
using the root guide's process/resource policy and already-present tooling.
Cargo build/check/test commands may use `--offline --locked` to preserve the
shared product resolution; command inventory is not a requirement to run every
suite for each change.

Default features are empty. Optional surfaces are `text-accessibility` and
`text-render`. The current text-render bridge has pre-existing API mismatches
against `surgeist-render`; do not assume it or an all-features combination passes.
Record that limitation separately from default/accessibility checks. Render
feature tests initialize a headless renderer and require a usable GPU backend.

```sh
cargo check -p surgeist-text
cargo test -p surgeist-text
cargo clippy -p surgeist-text --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-text --check
```
