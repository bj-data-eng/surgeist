# surgeist-css Local Guide

This crate is part of the Surgeist repository and product Cargo workspace.
Read [../../AGENTS.md](../../AGENTS.md) first for workflow, architecture, shared
verification/resource limits, and API generation. This supplement records local
facts; it grants no additional authority. The root guide prohibits executable
`unsafe` in all Surgeist-owned Rust.

This crate owns authored CSS syntax, intrinsic grammar validation, browser recovery,
diagnostic provenance, support metadata, and shared pure CSS language operations.
Ordinary authored parsing returns retained syntax and ordered diagnostics; the
always-available validators accept exactly clean reports without selecting another
grammar. Keep authored symbolic values unresolved. The separately selected
[standalone color producer](../../references/SOURCE-EDITIONS.md#html-color-delegation-and-intrinsic-contrast-policy)
is contribution work under [#805](https://github.com/bj-data-eng/surgeist/issues/805):
CSS owns source parsing, exact authored expressions, contextual resolution and
source/fact qualification over explicit inputs. The dependency-free
[`surgeist-color`](../surgeist-color/README.md) owner supplies numerical values,
conversion, interpolation, transformations, resolved serialization and finite
formatting. CSS retains exact authored decimal and specified serialization.
Current source and tests establish implemented capabilities; the checked-authored
used-color feeder remains unimplemented. Cascade, inheritance, substitution,
selector/query evaluation, resource retrieval, layout, painting and each consuming
crate's lowering remain outside this crate.

Consume sibling public front doors. Each consuming crate owns its direct adapter
to lower-level contracts, including HTML's current direct adapter to CSS source
operations and Color's numerical owner. The HTML values/parser/semantics split
remains planned.
The root facade owns optional host-facing composition; it is not a mandatory
integration intermediary. Follow the root guide's ownership and dependency rules.

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
| Shared handwritten property inventories and their consumers | `tests/common/property_expectations/README.md` and `records.rs` in that directory |
| Corpus provenance, generated set, and maintenance owner | `tests/corpus/csstree/README.md` and `tests/corpus/csstree/corpus.toml` |
| CSS-owned expectation and oracle contracts | `tests/csstree/`, `tests/support/csstree.rs`, and `tests/support/csstree_adapters.rs` |
| Project license and third-party attribution | `LICENSE`, `NOTICE.md`, and the local license files linked by the notice |

## Local Verification

Run the applicable commands from this crate directory, one package at a time,
using the root guide's process/resource policy and already-present tooling.
Cargo build/check/test commands may use `--offline --locked` to preserve the
shared product resolution; command inventory is not a requirement to run every
suite for each change.

Validation and parsing share the default public surface. Full package tests include corpus
parser probes. The [corpus guide](tests/corpus/csstree/README.md) owns maintenance
with the local `surgeist-generator` CLI; do not hand-edit its generated set.
`tests/csstree/expected-classes.json` is CSS-owned expectation data; `oracle.json`
is separately captured by the ignored environment-gated test in
`tests/csstree_baseline.rs`. Capture is a writing operation, not a routine check.

```sh
cargo check --offline -p surgeist-css
cargo test --offline -p surgeist-css
cargo test --offline -p surgeist-css --doc
cargo clippy --offline -p surgeist-css --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-css --check
```
