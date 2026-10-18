# surgeist-color Crate Guide

Read the [repository guide](../../AGENTS.md) for shared policy and workflow.

This crate owns checked numerical color/space values, intrinsic conversion,
interpolation, numerical transformations, resolved serialization, finite
formatting/rounding and byte-alpha serialization policy. It has no Cargo
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
refusal without ownership loss, explicit recovery, and the actual compiled
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

Conversion tests preserve extended/missing coordinates and final-overflow
behavior. Controlled-operation tests cover native use, actual guard lifetimes,
foreign digits, refusal, recovery, finite limits and emitted output. CSS and
HTML own cross-crate behavior tests. Existing fuzz harness maintenance stays with
its consumer; new fuzz adoption and campaigns remain deferred under #1172.
