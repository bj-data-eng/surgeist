# surgeist-generator Crate Guide

Read the [repository guide](../../AGENTS.md) for shared policy and workflow.
This supplement records crate facts and grants no additional authority.

## Product Boundary

This crate owns shared generation infrastructure, the `css-corpus` API/CLI, and the
caller-driven `browser-corpus` engine. Callers own their host executable, fixture
semantics, measurement protocol, and serialization; there is no layout-specific
API or binary here. The default value/read library is native/wasm portable;
mutation support requires Apple-Silicon macOS. Corpus locations enforce contained
roots; explicit acquisition uses owner-local `tmp/surgeist-sources` and
`tmp/surgeist-browser` caches. Existing-only operations and corpus checks do not
acquire software. Manifests/caller declarations own pins, paths, counts, and
browser settings. Production corpora belong to the consuming crates.

The CSS API imports either a pinned CSSTree checkout or a reviewed explicit WPT
source/license bundle. WPT import and checking remain offline and browser-free;
the consuming CSS corpus owns acquisition provenance and adaptation evidence.
Select [tests/wpt_import.rs](tests/wpt_import.rs) with
`--no-default-features --features css-corpus --test wpt_import`. The importer does not provide a WPT generation transform.

## Discover The Current Structure

| Fact | Authoritative source |
| --- | --- |
| Package identity, dependencies, features and targets | [Cargo.toml](Cargo.toml) |
| Public front door | [src/lib.rs](src/lib.rs) and its reexports |
| Current behavior and crate boundary | [src/](src/), [docs/reference.md](docs/reference.md), and [docs/explanation.md](docs/explanation.md) |
| Setup and operating procedures | [docs/getting-started.md](docs/getting-started.md) and [docs/how-to.md](docs/how-to.md) |
| Focused verification | tracked `#[cfg(test)]` modules in [src/](src/) and integration tests in [tests/](tests/) |
| Verification commands and dependency policy | This command inventory, Cargo targets/features in [Cargo.toml](Cargo.toml), and [deny.toml](deny.toml) |

[README.md](README.md) links usage, contract and attribution documentation.

## Command Inventory

Select affected checks under the root process/resource policy; commands below
run from this crate directory.

Mutation tests require Apple-Silicon macOS. Browser-feature tests own helper
processes and need separate bounded runs. The ignored command is list-only;
exhaustive diagnostics require explicit selection. License checks use this crate
as graph root and its [deny.toml](deny.toml); omit `--workspace`. Advisory checks
read the entire shared product lockfile without establishing workspace policy.

Browser fixtures hash the test executable. Set
`CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_TEST_STRIP=symbols` to reduce hashed bytes
while retaining debug assertions; allow a longer supervised command budget.

```sh
RUSTFLAGS="-D warnings" cargo check -j 1 --locked --offline -p surgeist-generator --no-default-features
RUSTFLAGS="-D warnings" cargo check -j 1 --locked --offline -p surgeist-generator --features browser-corpus
RUSTFLAGS="-D warnings" cargo check -j 1 --locked --offline -p surgeist-generator --features css-corpus
RUSTFLAGS="-D warnings" cargo check -j 1 --locked --offline -p surgeist-generator --all-features
cargo test -j 1 --locked --offline -p surgeist-generator --no-default-features
cargo test -j 1 --locked --offline -p surgeist-generator --features browser-corpus
cargo test -j 1 --locked --offline -p surgeist-generator --features css-corpus
cargo test -j 1 --locked --offline -p surgeist-generator --all-features
cargo test -j 1 --locked --offline -p surgeist-generator --all-features -- --ignored --list
cargo clippy -j 1 --locked --offline -p surgeist-generator --no-default-features --all-targets -- -F unsafe-code -D warnings
cargo clippy -j 1 --locked --offline -p surgeist-generator --features browser-corpus --all-targets -- -F unsafe-code -D warnings
cargo clippy -j 1 --locked --offline -p surgeist-generator --features css-corpus --all-targets -- -F unsafe-code -D warnings
cargo clippy -j 1 --locked --offline -p surgeist-generator --all-features --all-targets -- -F unsafe-code -D warnings
RUSTFLAGS="-D warnings" cargo check -j 1 --locked --offline -p surgeist-generator --target wasm32-unknown-unknown --no-default-features --lib
cargo metadata --locked --offline --no-deps --format-version 1
cargo fmt -p surgeist-generator -- --check
cargo deny --manifest-path Cargo.toml --all-features --locked --offline check --config deny.toml licenses
cargo audit --file ../../Cargo.lock --no-fetch --stale
```
