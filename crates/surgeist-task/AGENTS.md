# surgeist-task Crate Guide

Read the [repository guide](../../AGENTS.md) for shared policy and workflow.
This supplement records crate facts and grants no additional authority.

## Product Boundary

This crate owns task identity, scopes, lifecycle, cancellation, progress, observation,
scheduling/admission policy, event queues, executor contracts, and Tokio execution.
Source establishes which declared policies are operational. App reducers/effects,
UI/window integration, retained/native-wake bridges, app resource descriptors,
and mapping task events to app inputs remain outside this crate.

## Discover The Current Structure

| Fact | Authoritative source |
| --- | --- |
| Package identity, dependencies, features and targets | [Cargo.toml](Cargo.toml) |
| Public front door and exported surface | [src/lib.rs](src/lib.rs) and its reexports |
| Current task behavior and runtime semantics | Implementation and inline tests under [src/](src/); source-linked [docs/reference.md](docs/reference.md) and [docs/explanation.md](docs/explanation.md) |
| Minimal usage and first-success proof | [examples/basic_task.rs](examples/basic_task.rs) and [docs/getting-started.md](docs/getting-started.md) |
| Focused verification | Inline `#[cfg(test)]` modules, including [src/testing.rs](src/testing.rs) |

[README.md](README.md) links usage, contract and attribution documentation.

## Command Inventory

Select affected checks under the root process/resource policy; commands below
run from this crate directory.

The `basic_task` example is a local usage check. Tokio execution can create
internal workers even when Rust test threads are limited; follow root resource
limits and the package’s actual feature declarations.

```sh
cargo check -j 1 -p surgeist-task
cargo test -j 1 -p surgeist-task
cargo clippy -j 1 -p surgeist-task --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-task -- --check
cargo run -j 1 -p surgeist-task --example basic_task
```
