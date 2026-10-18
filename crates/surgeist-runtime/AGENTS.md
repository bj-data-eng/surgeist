# surgeist-runtime Crate Guide

Read the [repository guide](../../AGENTS.md) for shared policy and workflow.
This supplement records crate facts and grants no additional authority.

## Product Boundary

This crate owns app orchestration, events/effects, lifecycle, invalidation, frame
scheduling, and provenance. Domain parsers, resolvers, algorithms, backends, and
concrete task execution policy remain outside this crate. Task cancellation,
progress, admission, and Tokio execution belong to `surgeist-task`.

Consume sibling public front doors. Runtime owns its adapters to lower-level
contracts, including the canonical DOM host capture join; root owns workspace
wiring and facade composition.

## Discover The Current Structure

| Fact | Authoritative source |
| --- | --- |
| Package identity, dependencies, features and targets | [Cargo.toml](Cargo.toml) |
| Public front door and exported surface | [src/lib.rs](src/lib.rs) and its reexports |
| Behavior and domain boundary | [README.md](README.md) and implementation under [src/](src/) |
| Focused verification | [src/tests.rs](src/tests.rs), test modules in [src/resource.rs](src/resource.rs) and [src/coord.rs](src/coord.rs), and private fixtures in [src/testing.rs](src/testing.rs) |

[README.md](README.md) links usage, contract and attribution documentation.

## Command Inventory

Select affected checks under the root process/resource policy; commands below
run from this crate directory.

Focused evidence is in `src/tests.rs`, `src/resource.rs`, and `src/coord.rs`;
private fixtures live in `src/testing.rs`.

```sh
cargo check -j 1 -p surgeist-runtime
cargo test -j 1 -p surgeist-runtime
cargo clippy -j 1 -p surgeist-runtime --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-runtime -- --check
```
