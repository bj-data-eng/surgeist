# surgeist-generator Local Guide

This crate is part of the Surgeist repository and product Cargo workspace.
Read [../../AGENTS.md](../../AGENTS.md) first for workflow, architecture, shared
verification/resource limits, and API generation. This supplement records local
facts; it grants no additional authority. The root guide prohibits executable
`unsafe` in all Surgeist-owned Rust.

This crate owns shared generation infrastructure, the `css-corpus` API/CLI, and the
caller-driven `browser-corpus` engine. Callers own their host executable, fixture
semantics, measurement protocol, and serialization; there is no layout-specific
API or binary here. The default value/read library is native/wasm portable;
mutation support requires Apple-Silicon macOS. Corpus locations enforce contained
roots; explicit acquisition uses owner-local `tmp/surgeist-sources` and
`tmp/surgeist-browser` caches. Existing-only operations and corpus checks do not
acquire software. Manifests/caller declarations own pins, paths, counts, and
browser settings. Production corpora belong to the consuming crates.

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
| Package identity, edition, MSRV, dependencies, features, and targets | `Cargo.toml` |
| Public front door | `src/lib.rs` and its reexports |
| Adoption and documentation navigation | `README.md` |
| Current behavior and crate boundary | `src/`, `docs/reference.md`, and `docs/explanation.md` |
| Setup and operating procedures | `docs/getting-started.md` and `docs/how-to.md` |
| Focused verification | tracked `#[cfg(test)]` modules in `src/` and integration tests in `tests/` |
| Verification commands and dependency policy | This command inventory, Cargo targets/features in `Cargo.toml`, and `deny.toml` |
| Project license | `LICENSE` and `Cargo.toml` |
| Third-party attribution and coverage | `NOTICE.md`, `licenses/`, and exact upstream release material; `Cargo.toml` and [../../Cargo.lock](../../Cargo.lock) own dependency identity |

## Local Verification

Run the applicable commands from this crate directory, one package at a time,
using the root guide's process/resource policy and already-present tooling.
Cargo build/check/test commands may use `--offline --locked` to preserve the
shared product resolution; command inventory is not a requirement to run every
suite for each change.

Mutation tests require Apple-Silicon macOS. Browser-feature tests create
owned helper processes; run them separately under root resource limits. The
ignored command below is list-only; executing ignored exhaustive diagnostics
requires explicit selection. The license command uses this crate as the graph
root and explicitly selects its [deny.toml](deny.toml); omit `--workspace` so the
policy remains crate-local. The advisory command reads the shared product
lockfile and therefore reports the entire product resolution, not a filtered
generator graph; it does not establish a new workspace policy.

Browser-feature fixtures hash the running test executable. For bounded test
runs, set `CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_TEST_STRIP=symbols` to reduce
the bytes hashed while retaining debug assertions. These suites still need a
longer command budget than small library tests; keep process supervision and
the root resource limits in effect.

```sh
RUSTFLAGS="-D warnings" cargo check --locked --offline -p surgeist-generator --no-default-features
RUSTFLAGS="-D warnings" cargo check --locked --offline -p surgeist-generator --features browser-corpus
RUSTFLAGS="-D warnings" cargo check --locked --offline -p surgeist-generator --features css-corpus
RUSTFLAGS="-D warnings" cargo check --locked --offline -p surgeist-generator --all-features
cargo test --locked --offline -p surgeist-generator --no-default-features
cargo test --locked --offline -p surgeist-generator --features browser-corpus
cargo test --locked --offline -p surgeist-generator --features css-corpus
cargo test --locked --offline -p surgeist-generator --all-features
cargo test --locked --offline -p surgeist-generator --all-features -- --ignored --list
cargo clippy --locked --offline -p surgeist-generator --no-default-features --all-targets -- -F unsafe-code -D warnings
cargo clippy --locked --offline -p surgeist-generator --features browser-corpus --all-targets -- -F unsafe-code -D warnings
cargo clippy --locked --offline -p surgeist-generator --features css-corpus --all-targets -- -F unsafe-code -D warnings
cargo clippy --locked --offline -p surgeist-generator --all-features --all-targets -- -F unsafe-code -D warnings
RUSTFLAGS="-D warnings" cargo check --locked --offline -p surgeist-generator --target wasm32-unknown-unknown --no-default-features --lib
cargo metadata --locked --offline --no-deps --format-version 1
cargo fmt -p surgeist-generator --check
cargo deny --manifest-path Cargo.toml --all-features --locked --offline check --config deny.toml licenses
cargo audit --file ../../Cargo.lock --no-fetch --stale
```
