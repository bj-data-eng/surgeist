# surgeist-layout Crate Guide

Read the [repository guide](../../AGENTS.md) for shared policy and workflow.
This supplement records crate facts and grants no additional authority.

## Product Boundary

This crate owns normalized layout-ready inputs, layout algorithms, physical node and
inline-fragment geometry, scroll geometry, sizing, caches, transactional
computation, and parity/oracle tests. `FlowAxes` owns production writing-mode
mapping; public geometry remains physical and algorithm-local carriers private.
CSS parsing/style resolution, retained identity, text shaping, rendering, and
live scrolling remain outside this crate. The private fixture adapter is a
constrained test bridge, not another CSS front door.

## Discover The Current Structure

| Fact | Authoritative source |
| --- | --- |
| Package identity, dependencies, features and targets | [Cargo.toml](Cargo.toml) |
| Public front door and modules | [src/lib.rs](src/lib.rs) holds the four compute functions and declares the public modules; each module's own file lists its items: [tree](src/tree.rs), [scalar](src/scalar.rs), [geometry](src/geometry.rs), [sizing](src/sizing.rs), [cache](src/cache.rs), [error](src/error.rs), [flex](src/flex/mod.rs), [grid](src/grid/mod.rs), [scroll](src/scroll/mod.rs) with [anchor_selection](src/scroll/anchor_selection.rs), [inline](src/inline/mod.rs), [inline_wrapping](src/inline_wrapping/mod.rs), [text_layout](src/text_layout/mod.rs), [fragmentation](src/fragmentation/mod.rs), [resize_observer](src/resize_observer/mod.rs), and the files that only list items defined elsewhere: [baseline](src/baseline.rs), [batch](src/batch.rs), [box_model](src/box_model.rs), [constraint](src/constraint.rs), [length](src/length.rs), [node](src/node.rs), [overflow](src/overflow.rs), [request](src/request.rs), [text_flow](src/text_flow.rs) |
| Prepared Text and inline geometry | [Text protocol](../surgeist-text/src/protocol/mod.rs), [concrete capture/adapter](src/text_layout/mod.rs), and [box/control-only numeric fitter](src/inline/mod.rs); [supplied lines](../surgeist-text/src/supplied_line.rs) own canonical numeric ordering |
| Implemented behavior and module ownership | [src/](src/); human navigation in [README.md](README.md) and [docs/](docs/) |
| Host traversal, measurement, cache, and batch-application contracts | [src/tree.rs](src/tree.rs), [src/measurement.rs](src/measurement.rs), [src/cache.rs](src/cache.rs), [src/output.rs](src/output.rs), and [src/engine/](src/engine/) |
| Focused verification | Tracked test modules under [src/](src/), [src/test_support/](src/test_support/), and the [tests/layout.rs](tests/layout.rs) integration target |
| Task recipes and composed checks | [Justfile](Justfile), [scripts/run-cargo-task.sh](scripts/run-cargo-task.sh), [scripts/run-verification.sh](scripts/run-verification.sh), and [scripts/run-browser-parity-task.sh](scripts/run-browser-parity-task.sh) |
| Browser source pins, launch settings, fixture inventory, and live accounting | [source manifest](tests/layout/browser_parity/corpus.toml), [live declaration](tests/layout/browser_parity/live-corpus.toml) |
| Fixture procedures and private measurement protocol | [tests/layout/browser_parity/README.md](tests/layout/browser_parity/README.md) and [tests/layout/browser_parity/measurement-protocol.md](tests/layout/browser_parity/measurement-protocol.md) |
| Generator target and supported commands | [Cargo.toml](Cargo.toml), [tests/bin/surgeist-layout-generate.rs](tests/bin/surgeist-layout-generate.rs), and [tests/bin/surgeist-layout-generate/cli.rs](tests/bin/surgeist-layout-generate/cli.rs) |
| Optional audit catalog and its separate tooling | [tools/surgeist-layout-audits/README.md](tools/surgeist-layout-audits/README.md), [tools/surgeist-layout-audits/Cargo.toml](tools/surgeist-layout-audits/Cargo.toml), and [tools/surgeist-layout-audits/rust-toolchain.toml](tools/surgeist-layout-audits/rust-toolchain.toml) |

[README.md](README.md) links usage, contract and attribution documentation.

## Command Inventory

Select affected checks under the root process/resource policy; commands below
run from this crate directory.

The optional generator uses the local `surgeist-generator` browser engine.
Its binary/helper tests require Node.js; browser acquisition/generation/import
require Apple-Silicon macOS. `check-corpus` validates provenance/accounting without
a browser or source cache. Follow the [corpus guide](tests/layout/browser_parity/README.md)
for refreshes; never hand-edit generated XML/reports. `Justfile` and `scripts/`
retain composed recipes, including recipes that clear filters and overrides.
The [audit tool](tools/surgeist-layout-audits/README.md) retains a separate nightly
workspace and is opt-in. Ordinary parity tests require complete historical and
live accounting and current registered qualification. The strict raw verdict
uses the same inventory and reports every failure. Both use existing XML.
The two deep-nesting tests in `src/sizing.rs` each construct 100,000 nodes; isolate
them from an initial bounded test run under the root resource policy.

```sh
cargo check -j 1 --offline --locked -p surgeist-layout
cargo test -j 1 --offline --locked -p surgeist-layout
cargo clippy -j 1 --offline --locked -p surgeist-layout --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-layout -- --check
```

Release-mode benchmarks over deterministic generated documents are `benches/` targets that use no dependencies beyond the package's own; a name after `--` selects documents.

```sh
cargo bench --offline --locked -j 1 -p surgeist-layout --bench structure
cargo bench --offline --locked -j 1 -p surgeist-layout --bench text
```

```sh
env -u SURGEIST_PARITY_FILTER SURGEIST_PARITY_RAW_EXACT=1 cargo test -j 1 --offline --locked -p surgeist-layout --test layout layout::browser_parity::runs_all_checked_in_browser_parity_xml -- --exact
```

```sh
cargo test -j 1 --offline --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate
cargo run -j 1 --offline --locked -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- check-corpus
```
