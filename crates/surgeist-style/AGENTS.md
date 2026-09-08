# surgeist-style Local Guide

This crate is part of the Surgeist repository and product Cargo workspace.
Read [../../AGENTS.md](../../AGENTS.md) first for workflow, architecture, shared
verification/resource limits, and API generation. This supplement records local
facts; it grants no additional authority. The root guide prohibits executable
`unsafe` in all Surgeist-owned Rust.

This crate owns typed style values, cascade, resolution, validation, and invalidation.
CSS parsing and layout/text/render lowering remain outside this crate. Algorithms
that consume typed property values belong to their domain owners. The existing
`From<Color> for peniko::Color` conversion in `src/value.rs` is a known discrepancy
with the intended render-lowering boundary; preserve it during unrelated work
without treating it as a new ownership rule.

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
| Package identity, edition, declared compatibility, dependencies, features, and targets | [Cargo.toml](Cargo.toml) |
| Public front door and exported surface | [src/lib.rs](src/lib.rs) and its reexports |
| Adoption and documentation navigation | [README.md](README.md) and its four linked guides under `docs/` |
| Implemented behavior | `src/`; in particular `property.rs`, `declaration.rs`, `authored.rs`, `selector.rs`, `sheet.rs`, `resolver.rs`, and `invalidation.rs` |
| Host tree and contextual inputs | [src/tree.rs](src/tree.rs), `Context` in [src/resolver.rs](src/resolver.rs), and [src/condition.rs](src/condition.rs) |
| Focused verification | Tracked `#[cfg(test)]` modules under `src/`, [tests/type_safety.rs](tests/type_safety.rs), and its `tests/compile_fail/` and `tests/compile_pass/` fixtures |
| Project license and dependency attribution | [LICENSE](LICENSE), [NOTICE.md](NOTICE.md), and its linked material under `licenses/`; upstream package metadata and legal files establish dependency terms |

## Local Verification

Run the applicable commands from this crate directory, one package at a time,
using the root guide's process/resource policy and already-present tooling.
Cargo build/check/test commands may use `--offline --locked` to preserve the
shared product resolution; command inventory is not a requirement to run every
suite for each change.

The `.stderr` files under `tests/compile_fail/` are compiler-diagnostic
snapshots consumed by `tests/type_safety.rs`. Keep each paired with its Rust
fixture and require evidence from the owning test before changing snapshots.

```sh
cargo check -p surgeist-style
cargo test -p surgeist-style
cargo clippy -p surgeist-style --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-style --check
```
