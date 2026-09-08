# surgeist-css Local Guide

This crate is part of the Surgeist repository and product Cargo workspace.
Read [../../AGENTS.md](../../AGENTS.md) first for workflow, architecture, shared
verification/resource limits, and API generation. This supplement records local
facts; it grants no additional authority. The root guide prohibits executable
`unsafe` in all Surgeist-owned Rust.

This crate owns authored CSS syntax, intrinsic grammar validation, browser recovery,
diagnostic provenance, and support metadata. Ordinary parsing returns retained
syntax and ordered diagnostics; `app-strict` validates the same reports without
selecting another grammar. Keep symbolic values unresolved. Cascade, inheritance,
substitution, selector/query evaluation, resource loading, layout, painting, and
cross-crate lowering remain outside this crate.

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
| Public front door and feature gates | `src/lib.rs` and its reexports |
| Parsing and recovery behavior | `src/parser/`, `src/report.rs`, and `src/error.rs` |
| Authored syntax, source coordinates, and property schema | `src/syntax.rs`, `src/source.rs`, and `src/properties.rs` |
| Specification provenance, support, and exclusions | `src/conformance.rs` |
| Human documentation and API examples | `README.md` and `docs/` |
| Verification and public-contract evidence | Tracked test modules in `src/`, rustdoc examples, and `tests/` |
| Corpus provenance, generated set, and maintenance owner | `tests/corpus/csstree/README.md` and `tests/corpus/csstree/corpus.toml` |
| CSS-owned expectation and oracle contracts | `tests/csstree/`, `tests/support/csstree.rs`, and `tests/support/csstree_adapters.rs` |
| Project license and third-party attribution | `LICENSE`, `NOTICE.md`, and the local license files linked by the notice |

## Local Verification

Run the applicable commands from this crate directory, one package at a time,
using the root guide's process/resource policy and already-present tooling.
Cargo build/check/test commands may use `--offline --locked` to preserve the
shared product resolution; command inventory is not a requirement to run every
suite for each change.

Cover default and `app-strict` when affected. Full package tests include corpus
parser probes. The [corpus guide](tests/corpus/csstree/README.md) owns maintenance
with the local `surgeist-generator` CLI; do not hand-edit its generated set.
`tests/csstree/expected-classes.json` is CSS-owned expectation data; `oracle.json`
is separately captured by the ignored environment-gated test in
`tests/csstree_baseline.rs`. Capture is a writing operation, not a routine check.

```sh
cargo check --offline -p surgeist-css
cargo check --offline -p surgeist-css --features app-strict
cargo test --offline -p surgeist-css
cargo test --offline -p surgeist-css --features app-strict
cargo test --offline -p surgeist-css --doc
cargo test --offline -p surgeist-css --doc --features app-strict
cargo clippy --offline -p surgeist-css --all-targets -- -F unsafe-code -D warnings
cargo clippy --offline -p surgeist-css --all-targets --features app-strict -- -F unsafe-code -D warnings
cargo fmt -p surgeist-css --check
```
