# surgeist-shape Local Guide

This crate is part of the Surgeist repository and product Cargo workspace.
Read [../../AGENTS.md](../../AGENTS.md) first for workflow, architecture, shared
verification/resource limits, and API generation. This supplement records local
facts; it grants no additional authority. The root guide prohibits executable
`unsafe` in all Surgeist-owned Rust.

This crate owns resolved geometry, primitive paths, shape normalization, bounds,
containment, path conversion, geometry keys, stroke geometry, and dash geometry.
Constructors and validated transitions own domain invariants. Style resolution,
layout, GPU resources, widgets, and application behavior remain outside this crate.

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
| Package identity, edition, dependencies, features, and targets | `Cargo.toml` |
| Public front door and exported surface | `src/lib.rs` and its reexports |
| Adoption and documentation navigation | `README.md` and the four guides under `docs/` |
| Behavior and domain invariants | Implementation under `src/` and focused assertions in `src/tests.rs` |
| Focused verification | `src/tests.rs`, registered by `src/lib.rs` |
| Project licensing | `LICENSE` and the license declaration in `Cargo.toml` |
| Dependency attribution and included license material | `NOTICE.md` and its linked files under `licenses/`; compare with exact upstream versions when dependencies change |

## Local Verification

Run the applicable commands from this crate directory, one package at a time,
using the root guide's process/resource policy and already-present tooling.
Cargo build/check/test commands may use `--offline --locked` to preserve the
shared product resolution; command inventory is not a requirement to run every
suite for each change.

The manifest declares no feature switches. Focused assertions are in
`src/tests.rs`; preserve constructor and transition invariants.

```sh
cargo check -p surgeist-shape
cargo test -p surgeist-shape
cargo clippy -p surgeist-shape --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-shape --check
```
