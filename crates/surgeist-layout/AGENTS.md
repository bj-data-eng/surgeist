# surgeist-layout Local Guide

This crate is part of the Surgeist repository and product Cargo workspace.
Read [../../AGENTS.md](../../AGENTS.md) first for workflow, architecture, shared
verification/resource limits, and API generation. This supplement records local
facts; it grants no additional authority. The root guide prohibits executable
`unsafe` in all Surgeist-owned Rust.

This crate owns normalized layout-ready inputs, layout algorithms, physical node and
inline-fragment geometry, scroll geometry, sizing, caches, transactional
computation, and parity/oracle tests. `FlowAxes` owns production writing-mode
mapping; public geometry remains physical and algorithm-local carriers private.
CSS parsing/style resolution, retained identity, text shaping, rendering, and
live scrolling remain outside this crate. The private fixture adapter is a
constrained test bridge, not another CSS front door.

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
| Package identity, edition, MSRV, dependencies, features, targets, and package exclusions | `Cargo.toml`; exact resolved dependency versions in [../../Cargo.lock](../../Cargo.lock) |
| Public front door and reexports | `src/lib.rs` and the definitions it reexports |
| Implemented behavior and module ownership | `src/`; human navigation in `README.md` and `docs/` |
| Host traversal, measurement, cache, and batch-application contracts | `src/tree.rs`, `src/measurement.rs`, `src/cache.rs`, `src/output.rs`, and `src/engine/` |
| Focused verification | Tracked test modules under `src/`, `src/test_support/`, and the `tests/layout.rs` integration target |
| Task recipes and composed checks | `Justfile`, `scripts/run-cargo-task.sh`, `scripts/run-verification.sh`, and `scripts/run-browser-parity-task.sh` |
| Browser source pins, launch settings, fixture inventory, and declared accounting | `tests/layout/browser_parity/corpus.toml` |
| Fixture procedures and private measurement protocol | `tests/layout/browser_parity/README.md` and `tests/layout/browser_parity/measurement-protocol.md` |
| Generator target and supported commands | `Cargo.toml`, `tests/bin/surgeist-layout-generate.rs`, and `tests/bin/surgeist-layout-generate/cli.rs` |
| Optional audit catalog and its separate tooling | `tools/surgeist-layout-audits/README.md`, `tools/surgeist-layout-audits/Cargo.toml`, and `tools/surgeist-layout-audits/rust-toolchain.toml` |
| Project license and third-party attribution | `LICENSE`, `NOTICE.md`, and the local license material linked from the notice |

## Local Verification

Run the applicable commands from this crate directory, one package at a time,
using the root guide's process/resource policy and already-present tooling.
Cargo build/check/test commands may use `--offline --locked` to preserve the
shared product resolution; command inventory is not a requirement to run every
suite for each change.

The optional generator uses the local `surgeist-generator` browser engine.
Its binary/helper tests require Node.js; browser acquisition/generation/import
require Apple-Silicon macOS. `check-corpus` validates provenance/accounting without
a browser or source cache. Follow the [corpus guide](tests/layout/browser_parity/README.md)
for refreshes; never hand-edit generated XML/reports. `Justfile` and `scripts/`
retain composed recipes, including recipes that clear filters and overrides.
The [audit tool](tools/surgeist-layout-audits/README.md) retains a separate nightly
workspace and is opt-in. Full parity is ignored; execute it only when selected.
The two deep-nesting tests in `src/sizing.rs` each construct 100,000 nodes; isolate
them from an initial bounded test run under the root resource policy.

```sh
cargo check --offline --locked -p surgeist-layout
cargo test --offline --locked -p surgeist-layout
cargo clippy --offline --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-layout --check
```

```sh
env -u SURGEIST_PARITY_FILTER cargo test --offline --locked -p surgeist-layout --test layout runs_all_checked_in_browser_parity_xml -- --ignored
```

```sh
cargo test --offline --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate
cargo run --offline --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- check-corpus
```
