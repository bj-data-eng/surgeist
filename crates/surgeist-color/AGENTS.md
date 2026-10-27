# surgeist-color Crate Guide

Read the [repository guide](../../AGENTS.md) for shared policy and workflow.

This crate owns checked numerical color/space values, intrinsic conversion,
interpolation, numerical transformations, resolved serialization, finite
formatting/rounding and byte-alpha serialization policy. It also owns the
[`named`](src/named.rs) table of the 148 opaque Color 4 section 6.1 named colors
and its ASCII case-insensitive lookup: one dependency-free source for HTML's
legacy color recovery and CSS's keywords. The table is written from the retained
[specification text](../../references/css-color-4--CRD-css-color-4-20260908--bada647312a7.md#named-colors);
[its test](tests/named.rs) parses that text and compares every row, so change
the table only with a new selected edition. It has no Cargo
production or build dependencies, including optional or target-specific ones.
Standard-library use is permitted. Do not add CSS parsing, authored expression
resolution, source/fact identity or host policy here. CSS and other consumers own
their adapters and publication.

[Cargo.toml](Cargo.toml) owns package identity and dependencies;
[src/lib.rs](src/lib.rs) owns the public front door. Numerical policy and source
citations live with the conversion/serialization implementation and in the
[selected source editions](../../references/SOURCE-EDITIONS.md#html-color-delegation-and-intrinsic-contrast-policy).
The [README](README.md), [native example](examples/native.rs), inline tests and
[integration tests](tests/) demonstrate supported behavior.

Operations and caller-owned digits borrow the actual independent numerical
anchor. Preserve admission before numerical work, record access and output,
refusal without ownership loss, declining an action without effect (the
operation stays pending), and the actual compiled
layout descriptions. Reuse one numerical/formatting implementation for ordinary
and controlled APIs. Do not copy kernels into consumers.

## Command Inventory

Select affected checks under the root process/resource policy.

```sh
cargo test --offline --locked -j 1 -p surgeist-color
cargo clippy --offline --locked -j 1 -p surgeist-color --all-targets -- -F unsafe-code -D warnings
cargo run --offline --locked -j 1 -p surgeist-color --example native
cargo fmt -p surgeist-color -- --check
```

Conversion tests preserve extended/missing coordinates and closest-finite
overflow behavior. Controlled-operation tests cover native use, actual guard lifetimes,
foreign digits, refusal, declined actions, finite limits and emitted output. CSS and
HTML own cross-crate behavior tests. The [property tests](tests/property.rs)
generate proof-typed operands through `surgeist-fuzz`'s `property` feature, a
dev-dependency, and run under ordinary `cargo test`; this crate owns their
cases and codec. Fuzz campaigns are not adopted here.
