# surgeist-runtime Local Guide

This crate is part of the Surgeist repository and product Cargo workspace.
Read [../../AGENTS.md](../../AGENTS.md) first for workflow, architecture, shared
verification/resource limits, and API generation. This supplement records local
facts; it grants no additional authority. The root guide prohibits executable
`unsafe` in all Surgeist-owned Rust.

This crate owns app orchestration, events/effects, lifecycle, invalidation, frame
scheduling, and provenance. Domain parsers, resolvers, algorithms, backends, and
concrete task execution policy remain outside this crate. Task cancellation,
progress, admission, and Tokio execution belong to `surgeist-task`.

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
| Package identity, edition, MSRV, dependencies, features, and targets | `Cargo.toml` |
| Public front door and exported surface | `src/lib.rs` and its reexports |
| Behavior and domain boundary | `README.md` and implementation under `src/` |
| User documentation | `README.md` and the documentation pages it links |
| Focused verification | `src/tests.rs`, test modules in `src/resource.rs` and `src/coord.rs`, and private fixtures in `src/testing.rs` |

## Local Verification

Run the applicable commands from this crate directory, one package at a time,
using the root guide's process/resource policy and already-present tooling.
Cargo build/check/test commands may use `--offline --locked` to preserve the
shared product resolution; command inventory is not a requirement to run every
suite for each change.

Focused evidence is in `src/tests.rs`, `src/resource.rs`, and `src/coord.rs`;
private fixtures live in `src/testing.rs`.

```sh
cargo check -p surgeist-runtime
cargo test -p surgeist-runtime
cargo clippy -p surgeist-runtime --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-runtime --check
```
