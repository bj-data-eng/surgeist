# surgeist-retained Local Guide

This crate is part of the Surgeist repository and product Cargo workspace.
Read [../../AGENTS.md](../../AGENTS.md) first for workflow, architecture, shared
verification/resource limits, and API generation. This supplement records local
facts; it grants no additional authority. The root guide prohibits executable
`unsafe` in all Surgeist-owned Rust.

This crate owns retained identity/state, canonical topology, stable handles, projected
traversal, virtual materialization state, event routes, mutation reports, and
selector facts/invalidation. Layout, style, rendering, platform input, widget
implementations, and application command execution remain outside this crate.

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
| Package identity, version, edition, license declaration, repository URL, dependencies, features, and targets | `Cargo.toml` |
| Public front door and exported surface | `src/lib.rs` and its reexports |
| Human entry point and documentation navigation | `README.md` |
| Usage and retained-domain concepts | `docs/getting-started.md`, `docs/how-to.md`, `docs/reference.md`, and `docs/explanation.md`, backed by `src/` |
| Retained identity and authored elements | `src/identity.rs`, `src/element.rs`, and `src/string.rs` |
| Mutation, projection, state, and event behavior | `src/model.rs`, `src/transaction.rs`, and the types reexported by `src/lib.rs` |
| Snapshot and change-report contracts | `src/snapshot.rs` and `src/change.rs` |
| Focused verification | `src/tests.rs`, its inclusion in `src/lib.rs`, and compile-fail doctests in `src/state.rs` |
| Project license and third-party attribution coverage | `LICENSE`, `NOTICE.md`, and dependency declarations in `Cargo.toml` |

## Local Verification

Run the applicable commands from this crate directory, one package at a time,
using the root guide's process/resource policy and already-present tooling.
Cargo build/check/test commands may use `--offline --locked` to preserve the
shared product resolution; command inventory is not a requirement to run every
suite for each change.

Focused tests live in `src/tests.rs`; `src/state.rs` also has compile-fail
doctests. Derive feature/target coverage from the manifest.

```sh
cargo check -p surgeist-retained
cargo test -p surgeist-retained
cargo clippy -p surgeist-retained --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-retained --check
```
