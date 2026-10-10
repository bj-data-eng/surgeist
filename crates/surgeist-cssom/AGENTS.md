# CSSOM crate guide

Apply the root repository guide and focused PISCT skills. This crate owns live
authored CSS state, its CSS integration adapter, immutable captures and atomic
publication. CSS owns syntax/value grammar, contextual admission and formatting.
The user authorized the CSSOM adapter ownership exception. This crate does not
own root bindings, evaluation, cascade, layout or scheduling.

The intentional public front door is `src/lib.rs`; `README.md` records lifecycle,
quotas, source guards, current edit scope and downstream coverage. Integration
tests are public consumers, and `examples/snapshot_consumer.rs` is executable.
Keep current checked payloads separate from original source occurrences. Never
reconstruct edited selected declarations by expanding their original shorthands.

Focused checks, through bounded `pisct run` and a checkout-owned target:

```sh
cargo test --offline --locked -j 1 -p surgeist-cssom
cargo run --offline --locked -j 1 -p surgeist-cssom --example snapshot_consumer
cargo clippy --offline --locked -j 1 -p surgeist-cssom --all-targets -- -F unsafe-code -D warnings
cargo fmt --all -- --check
```
