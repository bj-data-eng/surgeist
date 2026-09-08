# surgeist-dialog Local Guide

This crate is part of the Surgeist repository and product Cargo workspace.
Read [../../AGENTS.md](../../AGENTS.md) first for workflow, architecture, shared
verification/resource limits, and API generation. This supplement records local
facts; it grants no additional authority. The root guide prohibits executable
`unsafe` in all Surgeist-owned Rust.

This crate owns dialog contracts, coordination primitives, and the native service
boundary. Application/window lifecycle and root orchestration remain outside
this crate. The optional `system` feature enables the native `rfd` dependency.

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
| Public front door | `src/lib.rs` and its reexports |
| Implemented behavior and crate boundary | `README.md` and `src/` |
| Focused verification | Tracked `#[cfg(test)]` modules and, when present, `tests/` and fixtures |

## Local Verification

Run the applicable commands from this crate directory, one package at a time,
using the root guide's process/resource policy and already-present tooling.
Cargo build/check/test commands may use `--offline --locked` to preserve the
shared product resolution; command inventory is not a requirement to run every
suite for each change.

Cover `--features system` separately when the native service boundary changes;
its platform/tooling requirements follow `Cargo.toml` and `src/lib.rs`.

```sh
cargo check -p surgeist-dialog
cargo test -p surgeist-dialog
cargo clippy -p surgeist-dialog --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-dialog --check
```
