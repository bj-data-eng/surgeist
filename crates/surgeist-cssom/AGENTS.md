# surgeist-cssom Crate Guide

Read the [repository guide](../../AGENTS.md) for shared policy and workflow.
[Cargo.toml](Cargo.toml), [src/lib.rs](src/lib.rs) and [README.md](README.md)
own package facts, the public front door and capability/verification guidance.

This crate owns live authored CSS state, its CSS integration adapter, immutable captures and atomic
publication. CSS owns syntax/value grammar, contextual admission and formatting.
This crate does not own root bindings, evaluation, cascade, layout or scheduling.

[README.md](README.md) records lifecycle, quotas, source guards and edit scope.
Integration tests are public consumers; [examples/snapshot_consumer.rs](examples/snapshot_consumer.rs)
is executable.
Keep current checked payloads separate from original source occurrences. Never
reconstruct edited selected declarations by expanding their original shorthands.

## Command Inventory

Select affected checks under the root process/resource policy:

```sh
cargo test --offline --locked -j 1 -p surgeist-cssom
cargo run --offline --locked -j 1 -p surgeist-cssom --example snapshot_consumer
cargo clippy --offline --locked -j 1 -p surgeist-cssom --all-targets -- -F unsafe-code -D warnings
cargo fmt --all -- --check
```
