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

## Stepwise Protocol Facts

`input::InputCursor` and `lexer::protocol::TokenCursor` are host-only sans-IO
protocols: a host admits each request, owns every bank and may refuse an
append, and the cursors stay unchanged and resumable. Their sizes, and those
of `Scalar`, `Record` and the types inside them, are pinned in
[tests/layout_pins.rs](tests/layout_pins.rs) because hosts charge them (the
`scratch` amount of surgeist-css's UTF-16 characterization rows includes
`size_of::<InputCursor>()`). Do not change a pinned layout; keep new state
outside these types.

The cursors read preprocessed scalars (Syntax 3 section 3.3) and use the
[lexical](src/lexical.rs) predicates, which describe preprocessed input. Text
that has not been preprocessed is preprocessed per code point before asking a
predicate.

## Command Inventory

Select affected checks under the root process/resource policy, from repository
root:

```sh
cargo check --offline --locked -j 1 -p surgeist-css-parser
cargo test --offline --locked -j 1 -p surgeist-css-parser
cargo clippy --offline --locked -j 1 -p surgeist-css-parser --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-css-parser -- --check
```
