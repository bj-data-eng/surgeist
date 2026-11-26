# surgeist-layout-parity Crate Guide

Read the [repository guide](../../AGENTS.md) for shared policy and workflow.
This supplement records crate facts and grants no additional authority.

## Product Boundary

This crate owns the browser-parity evidence for `surgeist-layout`: the checked-in
corpus of browser-captured XML and its reports, the golden generator, the
qualification driver, and the test support that compares layout output with the
captured geometry. It depends on `surgeist-layout`'s public API only; layout never
depends on this crate, and nothing here reaches layout's private items. The shared
browser engine and corpus mechanics belong to `surgeist-generator`; this crate owns
the layout host executable, fixture semantics, measurement protocol and
serialization. The root facade uses the crate as a development dependency for one
example. Layout fixtures that the layout tests read live in `surgeist-layout`.

## Discover The Current Structure

| Fact | Authoritative source |
| --- | --- |
| Package identity, dependencies, the `browser-corpus` feature and targets | [Cargo.toml](Cargo.toml) |
| Public surface and module inventory | [src/lib.rs](src/lib.rs), [src/browser_parity_modules.rs](src/browser_parity_modules.rs) |
| Golden parsing, the layout adapter and exact comparison | [src/browser_parity/](src/browser_parity/) with root [src/browser_parity.rs](src/browser_parity.rs) |
| Registered-edition admission | [src/browser_parity/portable_qualification.rs](src/browser_parity/portable_qualification.rs) |
| Generator commands, measurement and XML serialization | [src/generate.rs](src/generate.rs) and [src/generate/](src/generate/) |
| Qualification modes and publication | [src/qualify.rs](src/qualify.rs) and [src/qualify/](src/qualify/) |
| Executables | [src/bin/surgeist-layout-generate.rs](src/bin/surgeist-layout-generate.rs) and [examples/qualify-root-context.rs](examples/qualify-root-context.rs) call the library |
| Corpus, source pins and live accounting | [tests/layout/browser_parity/](tests/layout/browser_parity/), its [source manifest](tests/layout/browser_parity/corpus.toml) and [live declaration](tests/layout/browser_parity/live-corpus.toml) |
| Corpus procedures and private measurement protocol | [corpus guide](tests/layout/browser_parity/README.md) and [measurement protocol](tests/layout/browser_parity/measurement-protocol.md) |
| Measurement scripts, request files and their Node tests | [tests/bin/surgeist-layout-generate/](tests/bin/surgeist-layout-generate/) |
| Qualification inputs and per-edition procedures | [examples/qualify-root-context/](examples/qualify-root-context/) and [semantic/](semantic/) |
| Fixture copies and their provenance | [tests/fixtures/](tests/fixtures/) |
| Task recipes | [Justfile](Justfile), [scripts/run-cargo-task.sh](scripts/run-cargo-task.sh), [scripts/run-verification.sh](scripts/run-verification.sh) and [scripts/run-browser-parity-task.sh](scripts/run-browser-parity-task.sh) |

[README.md](README.md) links the corpus and attribution documentation.

## Layout Of The Crate

All Rust sources are in `src/`. The library holds the parity support, the
generator and the qualification driver; the binary and the example are one-line
callers, so each test runs in the library's unit-test target only. The generator
and the qualification driver (`generate`, `qualify`, `run_generator`,
`run_qualification`, the binary and the example) need the `browser-corpus`
feature; the default build serves the root facade without the browser engine. Tests of the
private support sit in `*_tests.rs` modules beside their owners. The strict raw
verdict is `browser_parity::tests::runs_all_checked_in_browser_parity_xml`.

The directories `tests/layout/browser_parity`, `tests/bin/surgeist-layout-generate`,
`examples/qualify-root-context` and `semantic` hold corpus data, scripts and
guides, not Rust. Corpus reports and manifests record these crate-relative paths,
so the directories keep their names. Never hand-edit generated XML or reports.

## Command Inventory

Select affected checks under the root process/resource policy; commands below
run from this crate directory.

Ordinary parity tests require complete historical and live accounting and
current registered qualification. They read existing XML and need no browser.
The strict raw verdict uses the same inventory and reports every failure.
`check-corpus` validates provenance and accounting without a browser or source
cache. The generator's helper tests require Node.js; browser acquisition,
generation and import require Apple-Silicon macOS. The qualification driver also
needs the cached browser. `Justfile` and `scripts/` retain composed recipes,
including recipes that clear filters and overrides.

The registered qualification reads one file outside this crate, the repository's
[text layout contract](../../docs/text-layout-contract.md), because the recorded
owner pin names that path and compares the current policy section with the retained
digest.

```sh
cargo check -j 1 --offline --locked -p surgeist-layout-parity --all-targets
cargo check -j 1 --offline --locked -p surgeist-layout-parity --features browser-corpus --all-targets
cargo test -j 1 --offline --locked -p surgeist-layout-parity
cargo test -j 1 --offline --locked -p surgeist-layout-parity --features browser-corpus
cargo clippy -j 1 --offline --locked -p surgeist-layout-parity --all-targets -- -F unsafe-code -D warnings
cargo clippy -j 1 --offline --locked -p surgeist-layout-parity --features browser-corpus --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-layout-parity -- --check
```

```sh
env -u SURGEIST_PARITY_FILTER SURGEIST_PARITY_RAW_EXACT=1 cargo test -j 1 --offline --locked -p surgeist-layout-parity --lib browser_parity::tests::runs_all_checked_in_browser_parity_xml -- --exact
```

```sh
cargo run -j 1 --offline --locked -p surgeist-layout-parity --features browser-corpus --bin surgeist-layout-generate -- check-corpus
cargo run -j 1 --offline --locked -p surgeist-layout-parity --features browser-corpus --bin surgeist-layout-generate -- check-taffy-corpus
```
