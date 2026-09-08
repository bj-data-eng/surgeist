# surgeist-animation Local Guide

This crate is part of the Surgeist repository and product Cargo workspace.
Read [../../AGENTS.md](../../AGENTS.md) first for workflow, architecture, shared
verification/resource limits, and API generation. This supplement records local
facts; it grants no additional authority. The root guide prohibits executable
`unsafe` in all Surgeist-owned Rust.

This crate owns timing, easing, keyframes, interpolation, sampled values, and animation
diagnostics. Root/style supply normalized tracks, computed endpoints, and
property support policy. Authored CSS, clocks, scheduling, lifecycle, invalidation,
color conversion, gamut mapping, and final property clamping remain caller-owned.

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
| Package identity, edition, dependencies, features, and targets | `Cargo.toml` |
| Public front door and reexports | `src/lib.rs` |
| Implemented behavior and focused verification | `src/`, its tracked test modules and doctests, and external consumer tests in `tests/` |
| Adoption and documentation navigation | `README.md` |
| First sample and usage procedures | `docs/getting-started.md`, `docs/how-to.md` |
| Public contract and capability descriptions | `docs/reference.md`, checked against `src/` |
| Ownership rationale and integration boundary | `docs/explanation.md` |
| Project license | `LICENSE`, with the declaration in `Cargo.toml` |
| Third-party attribution coverage | `NOTICE.md`, checked against the manifest and distributed material |
| Performance summaries and measurement commands | `docs/performance.md`, `docs/construction-performance.md`, `benches/`, `examples/allocation_check.rs`, and `performance_support/` |

## Local Verification

Run the applicable commands from this crate directory, one package at a time,
using the root guide's process/resource policy and already-present tooling.
Cargo build/check/test commands may use `--offline --locked` to preserve the
shared product resolution; command inventory is not a requirement to run every
suite for each change.

The manifest declares no features or runtime dependencies. Performance and
allocation commands are separate measurements, not routine test gates; their
contracts and commands are in `docs/performance.md` and
`docs/construction-performance.md`.

```sh
cargo check --offline -p surgeist-animation
cargo test --offline -p surgeist-animation
cargo test --offline -p surgeist-animation --doc
cargo doc --offline -p surgeist-animation --no-deps
cargo clippy --offline -p surgeist-animation --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-animation --check
cargo bench --offline -p surgeist-animation --bench sampling
cargo run --offline --release -p surgeist-animation --example allocation_check -- --check-values
cargo run --offline --release -p surgeist-animation --example allocation_check -- --check-failures
cargo run --offline --release -p surgeist-animation --example allocation_check -- --check-reuse
cargo run --offline --release -p surgeist-animation --example allocation_check -- --check-construction
cargo run --offline --release -p surgeist-animation --example allocation_check -- --check-intake
cargo run --offline --release -p surgeist-animation --example allocation_check -- --report-memory
```
