# surgeist-task Local Guide

This crate is part of the Surgeist repository and product Cargo workspace.
Read [../../AGENTS.md](../../AGENTS.md) first for workflow, architecture, shared
verification/resource limits, and API generation. This supplement records local
facts; it grants no additional authority. The root guide prohibits executable
`unsafe` in all Surgeist-owned Rust.

This crate owns task identity, scopes, lifecycle, cancellation, progress, observation,
scheduling/admission policy, event queues, executor contracts, and Tokio execution.
Source establishes which declared policies are operational. App reducers/effects,
UI/window integration, retained/native-wake bridges, app resource descriptors,
and mapping task events to app inputs remain outside this crate.

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
| Adoption and documentation navigation | `README.md` |
| Current task behavior and runtime semantics | Implementation and inline tests under `src/`; source-linked `docs/reference.md` and `docs/explanation.md` |
| Minimal usage and first-success proof | `examples/basic_task.rs` and `docs/getting-started.md` |
| Goal-oriented usage | `docs/how-to.md` and its linked source |
| Focused verification | Inline `#[cfg(test)]` modules, including `src/testing.rs` |
| Project licensing | `LICENSE` and `Cargo.toml` |
| Dependency attribution coverage and copied upstream terms | `NOTICE.md`, `licenses/`, and the exact dependency metadata and upstream versions identified there |

## Local Verification

Run the applicable commands from this crate directory, one package at a time,
using the root guide's process/resource policy and already-present tooling.
Cargo build/check/test commands may use `--offline --locked` to preserve the
shared product resolution; command inventory is not a requirement to run every
suite for each change.

The `basic_task` example is a local usage check. Tokio execution can create
internal workers even when Rust test threads are limited; follow root resource
limits and the package’s actual feature declarations.

```sh
cargo check -p surgeist-task
cargo test -p surgeist-task
cargo clippy -p surgeist-task --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-task --check
cargo run -p surgeist-task --example basic_task
```
