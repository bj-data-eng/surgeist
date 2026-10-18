# surgeist-dialog Crate Guide

Read the [repository guide](../../AGENTS.md) for shared policy and workflow.
This supplement records crate facts and grants no additional authority.

## Product Boundary

This crate owns dialog contracts, coordination primitives, and the native service
boundary. Application/window lifecycle and root orchestration remain outside
this crate. The optional `system` feature enables the native `rfd` dependency.

## Discover The Current Structure

| Fact | Authoritative source |
| --- | --- |
| Package identity, dependencies, features and targets | [Cargo.toml](Cargo.toml) |
| Public front door | [src/lib.rs](src/lib.rs) and its reexports |
| Implemented behavior and crate boundary | [README.md](README.md) and [src/](src/) |
| Focused verification | Tracked `#[cfg(test)]` modules and, when present, `tests/` and fixtures |

[README.md](README.md) links usage, contract and attribution documentation.

## Command Inventory

Select affected checks under the root process/resource policy; commands below
run from this crate directory.

Cover `--features system` separately when the native service boundary changes;
its platform/tooling requirements follow `Cargo.toml` and `src/lib.rs`.

```sh
cargo check -j 1 -p surgeist-dialog
cargo test -j 1 -p surgeist-dialog
cargo clippy -j 1 -p surgeist-dialog --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-dialog -- --check
```
