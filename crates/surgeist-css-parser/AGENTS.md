# surgeist-css-parser Crate Guide

Read the [repository guide](../../AGENTS.md) for shared policy and workflow.
This supplement records crate facts and grants no additional authority.

## Product Boundary

This crate owns CSS Syntax Module Level 3: input decoding and preprocessing,
tokenization, parsing results and token streams, parser entry points and
algorithms, the `An+B` microsyntax and token serialization. It gives syntax no
CSS meaning. [surgeist-css](../surgeist-css/AGENTS.md) owns grammars, property
and descriptor values, rules, selectors, queries and recovery diagnostics, and
depends on this crate. This crate depends on no Surgeist crate; do not add one.
Behavior follows the
[pinned CSS Syntax 3 edition](../../references/SOURCE-EDITIONS.md#css-syntax-3-edition),
not the conventions of another implementation.

## Discover The Current Structure

| Fact | Authoritative source |
| --- | --- |
| Package identity, dependencies and features | [Cargo.toml](Cargo.toml) |
| Public front door and module list | [src/lib.rs](src/lib.rs) |
| Clauses covered, their tests and what is not claimed | [docs/conformance-coverage.md](docs/conformance-coverage.md) |
| Crate purpose, status and edition pointer | [README.md](README.md) |

## Command Inventory

Select affected checks under the root process/resource policy, from repository
root:

```sh
cargo check --offline --locked -j 1 -p surgeist-css-parser
cargo test --offline --locked -j 1 -p surgeist-css-parser
cargo clippy --offline --locked -j 1 -p surgeist-css-parser --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-css-parser -- --check
```
