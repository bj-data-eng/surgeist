# surgeist-animation Crate Guide

Read the [repository guide](../../AGENTS.md) for shared policy and workflow.
This supplement records crate facts and grants no additional authority.

## Product Boundary

This crate owns timing, easing, keyframes, interpolation, sampled values, and animation
diagnostics. Consumers supply normalized tracks, computed endpoints, and
property support policy. Authored CSS, clocks, scheduling, lifecycle, invalidation,
color conversion, gamut mapping, and final property clamping remain caller-owned.

## Discover The Current Structure

| Fact | Authoritative source |
| --- | --- |
| Package identity, dependencies, features and targets | [Cargo.toml](Cargo.toml) |
| Public front door and reexports | [src/lib.rs](src/lib.rs) |
| Implemented behavior and focused verification | [src/](src/), its tracked test modules and doctests, and external consumer tests in [tests/](tests/) |
| First sample and usage procedures | [docs/getting-started.md](docs/getting-started.md), [docs/how-to.md](docs/how-to.md) |
| Ownership rationale and integration boundary | [docs/explanation.md](docs/explanation.md) |
| Performance summaries and measurement commands | [docs/performance.md](docs/performance.md), [docs/construction-performance.md](docs/construction-performance.md), [benches/](benches/), [examples/allocation_check.rs](examples/allocation_check.rs), and [performance_support/](performance_support/) |

[README.md](README.md) links usage, contract and attribution documentation.

## Command Inventory

Select affected checks under the root process/resource policy; commands below
run from this crate directory.

The manifest declares no features or runtime dependencies. Performance and
allocation commands are separate measurements, not routine test gates; their
contracts and commands are in `docs/performance.md` and
`docs/construction-performance.md`.

```sh
cargo check -j 1 --offline -p surgeist-animation
cargo test -j 1 --offline -p surgeist-animation
cargo test -j 1 --offline -p surgeist-animation --doc
cargo doc -j 1 --offline -p surgeist-animation --no-deps
cargo clippy -j 1 --offline -p surgeist-animation --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-animation -- --check
cargo bench -j 1 --offline -p surgeist-animation --bench sampling
cargo run -j 1 --offline --release -p surgeist-animation --example allocation_check -- --check-values
cargo run -j 1 --offline --release -p surgeist-animation --example allocation_check -- --check-failures
cargo run -j 1 --offline --release -p surgeist-animation --example allocation_check -- --check-reuse
cargo run -j 1 --offline --release -p surgeist-animation --example allocation_check -- --check-construction
cargo run -j 1 --offline --release -p surgeist-animation --example allocation_check -- --check-intake
cargo run -j 1 --offline --release -p surgeist-animation --example allocation_check -- --report-memory
```
