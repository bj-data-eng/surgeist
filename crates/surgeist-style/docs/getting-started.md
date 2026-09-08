# Getting started

Run one existing library test to see typed declarations become a resolved style.
The test attaches margin, padding, and border declarations to a `button` rule,
matches that rule against a small tree, and reads the resulting edge values.

## Prerequisites

- A checkout of this repository and a terminal at its root, beside
  [Cargo.toml](../Cargo.toml).
- An installed Rust/Cargo toolchain capable of building this edition-2024 package
  and its dependencies. The leaf manifest does not declare a minimum Rust version.
- The package's dependency graph available in the local Cargo cache. The manifest
  declares `peniko` as a dependency and `trybuild` as a development dependency.

The command below uses offline mode. If Cargo reports a missing cached package,
the dependency prerequisite is unmet; this command does not download it.

## Resolve a style

1. Open the existing
   [`resolved_edge_getters_assemble_side_longhands` test](../src/resolver.rs#L2569).
   Its `resolve_single` helper constructs a sheet containing a `button` selector
   and passes a matching node to `Resolver::resolve`.
2. Run that test from the crate directory:

   ```sh
   cargo test --offline -p surgeist-style --lib resolver::tests::resolved_edge_getters_assemble_side_longhands -- --exact
   ```

3. Verify that Cargo reports one executed test, that exact test name followed by
   `ok`, and no failed tests. Zero executed tests is not the expected result.

The assertions check a top margin of `2px`, a right margin of `4px`, a bottom
padding of `6px`, and a left border width of `8px`. Unset margin sides remain zero.
This demonstrates style resolution; it does not calculate a box layout or render
pixels.

Continue with [how-to guides](how-to.md) to supply your own tree, inspect
diagnostics, and compare resolved styles. The [reference](reference.md) maps the
public interfaces and verification commands to their source owners.
