# surgeist-shape Crate Guide

Read the [repository guide](../../AGENTS.md) for shared policy and workflow.
This supplement records crate facts and grants no additional authority.

## Product Boundary

This crate owns resolved geometry, primitive paths, shape normalization, bounds,
containment, path conversion, geometry keys, stroke geometry, and dash geometry.
Constructors and validated transitions own domain invariants. Style resolution,
layout, GPU resources, widgets, and application behavior remain outside this crate.

## Discover The Current Structure

| Fact | Authoritative source |
| --- | --- |
| Package identity, dependencies, features and targets | [Cargo.toml](Cargo.toml) |
| Public front door and exported surface | [src/lib.rs](src/lib.rs) and its reexports |
| Behavior and domain invariants | Implementation under [src/](src/) and focused assertions in [src/tests.rs](src/tests.rs) |
| Focused verification | [src/tests.rs](src/tests.rs), registered by [src/lib.rs](src/lib.rs) |

[README.md](README.md) links usage, contract and attribution documentation.

## Command Inventory

Select affected checks under the root process/resource policy; commands below
run from this crate directory.

The manifest declares no feature switches. Focused assertions are in
`src/tests.rs`; preserve constructor and transition invariants.

```sh
cargo check -j 1 -p surgeist-shape
cargo test -j 1 -p surgeist-shape
cargo clippy -j 1 -p surgeist-shape --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-shape -- --check
```
