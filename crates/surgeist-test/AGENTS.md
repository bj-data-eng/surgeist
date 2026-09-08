# surgeist-test Local Guide

This crate is part of the Surgeist repository and product Cargo workspace.
Read [../../AGENTS.md](../../AGENTS.md) first for workflow, architecture, shared
verification/resource limits, and API generation. This supplement records local
facts; it grants no additional authority. The root guide prohibits executable
`unsafe` in all Surgeist-owned Rust.

This crate provides test-facing schemas, harnesses, fixtures, and verification support.
Current layout-ready metadata schemas are independent of layout implementation.
Keep production behavior and root adapters outside this crate; the root facade
must not production-depend on it.

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
| Package identity, edition, dependencies, features, and targets | [Cargo.toml](Cargo.toml) |
| Public front door and reexports | [src/lib.rs](src/lib.rs), [src/fixtures/mod.rs](src/fixtures/mod.rs) |
| Fixture schema, constructors, and validation | [src/fixtures/layout_ready.rs](src/fixtures/layout_ready.rs) |
| Focused verification | The `#[cfg(test)]` module in [src/fixtures/mod.rs](src/fixtures/mod.rs) |
| Human entry point and fixture ownership explanation | [README.md](README.md), [docs/explanation.md](docs/explanation.md) |
| Project license and source-distribution attribution | [LICENSE](LICENSE), [NOTICE.md](NOTICE.md), with dependency inventory from [Cargo.toml](Cargo.toml) |

## Local Verification

Run the applicable commands from this crate directory, one package at a time,
using the root guide's process/resource policy and already-present tooling.
Cargo build/check/test commands may use `--offline --locked` to preserve the
shared product resolution; command inventory is not a requirement to run every
suite for each change.

Focused fixture-schema tests are in `src/fixtures/mod.rs`. Derive feature
and target coverage from the current manifest.

```sh
cargo check --offline -p surgeist-test
cargo test --offline -p surgeist-test
cargo clippy --offline -p surgeist-test --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-test --check
```
