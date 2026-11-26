# Reference

This page describes the crate's targets, the browser-parity runtime and the local
commands. The [manifest](../Cargo.toml), the [library root](../src/lib.rs) and the
linked source files own their current definitions.

## Package and targets

| Item | Current value |
| --- | --- |
| Package / library | `surgeist-layout-parity` / `surgeist_layout_parity` |
| Version | `0.1.0` |
| Rust edition / minimum version | `2024` / `1.99.1` |
| Features | None |
| Generator binary | `surgeist-layout-generate` |
| Qualification example | `qualify-root-context` |
| Publication | Not published |

The library holds the parity support, the generator and the qualification driver.
The binary and the example call it. Its public surface is the registered-edition
admission in `portable_qualification`, the pinned Ahem font bytes, the root of the
prepared atomic percentage edition, and the entry points of the two executables.
The path/version requirements are in the manifest; resolved dependencies are in
the shared [Cargo.lock](../../../Cargo.lock).

## Browser parity runtime

The `surgeist-layout-generate` binary supplies a private layout adapter to the
local `surgeist-generator` workspace dependency. The shared crate owns acquisition,
browser processes, input protection, reports, and atomic publication. This crate
owns helper preparation, measurement decoding, unsupported classification, and XML
serialization.

The corpus pins its Chrome-for-Testing version, launch settings, and report
inventory in [the corpus manifest](../tests/layout/browser_parity/corpus.toml). `generate` may acquire the
managed browser; `generate-existing` requires `SURGEIST_BROWSER_PATH` to name a
repository-relative executable below the declared cache and never fetches.
Source checkouts live in `tmp/surgeist-sources/<id>/<revision>/`; browsers live in
`tmp/surgeist-browser/`. Repository `tmp/` is ignored and survives `cargo clean`.

`check-corpus` validates the live schema-4 report, ownership, and persisted
source-import attestation without a browser or source checkout. The producer
publishes under `live-xml/` using the shared source manifest and live counts.
Explicit `check-taffy-corpus`
verifies the existing pinned source cache; `import-taffy` may acquire that source
and publishes verified imports plus attestation. Full generation validates
accounting before publishing. Filtered generation does not rewrite full reports.
XML bytes are comment-free; semantic parsing and comparison remain in the parity
support. See the [browser parity guide](../tests/layout/browser_parity/README.md)
for commands, cache behavior, and current provenance requirements.

`cargo test -p surgeist-layout-parity` traverses historical XML, registered
current captures, and live XML. It requires complete historical and live
accounting and fresh registered qualification. The live census compares all
generated captures through Native Layout and reports its failures separately.
The named strict raw verdict uses the same inventory and fails on every raw
mismatch or authority refusal. The generator's helper tests require Node.js.

## Local commands

Run these recipes from the crate directory with `just` already available. The
[Justfile](../Justfile), [Cargo task script](../scripts/run-cargo-task.sh),
[verification script](../scripts/run-verification.sh), and
[browser-parity task script](../scripts/run-browser-parity-task.sh) define them.
Setting `CARGO_NET_OFFLINE=true` keeps Cargo dependency resolution offline.

| Recipe | Observable operation |
| --- | --- |
| `just fmt` / `just fmt-check` | Format Rust / check Rust formatting |
| `just check` / `just test` / `just clippy` | Cargo check, tests, or strict Clippy over the crate |
| `just verify` | Formatting, check, test, and Clippy sequence |
| `just parity-all` | Traverse the full checked-in corpus and enforce the strict raw verdict |
| `just corpus-check` | Browser-free `check-corpus` with corpus and browser overrides cleared |
| `just taffy-check` | Validate the existing pinned source cache and imported Taffy corpus |

Clippy recipes use `-F unsafe-code -D warnings`; Cargo recipes use `-j 1 --offline
--locked`. Browser commands and environment variables are described in the
[browser parity guide](../tests/layout/browser_parity/README.md).
