# surgeist-test Crate Guide

Read the [repository guide](../../AGENTS.md) for shared policy and workflow.
This supplement records crate facts and grants no additional authority.

## Product Boundary

This crate provides test-facing schemas, harnesses, fixtures, and verification support.
Current layout-ready metadata schemas are independent of layout implementation.
Keep production behavior and root adapters outside this crate; the root facade
must not production-depend on it.

## Discover The Current Structure

| Fact | Authoritative source |
| --- | --- |
| Package identity, dependencies, features and targets | [Cargo.toml](Cargo.toml) |
| Public front door and reexports | [src/lib.rs](src/lib.rs), [src/fixtures/mod.rs](src/fixtures/mod.rs) |
| Fixture schema, constructors, and validation | [src/fixtures/layout_ready.rs](src/fixtures/layout_ready.rs) |
| Focused verification | The `#[cfg(test)]` module in [src/fixtures/mod.rs](src/fixtures/mod.rs) |

[README.md](README.md) links usage, contract and attribution documentation.

## Command Inventory

Select affected checks under the root process/resource policy; commands below
run from this crate directory.

Focused fixture-schema tests are in `src/fixtures/mod.rs`. Derive feature
and target coverage from the current manifest.

```sh
cargo check -j 1 --offline -p surgeist-test
cargo test -j 1 --offline -p surgeist-test
cargo clippy -j 1 --offline -p surgeist-test --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-test -- --check
```
