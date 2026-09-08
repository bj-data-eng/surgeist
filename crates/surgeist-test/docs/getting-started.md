# Getting started

Run the smallest existing test that demonstrates a layout-ready fixture record.
Success confirms the metadata constructors and accessors agree with the fixture
schema; it does not run a layout engine or generate fixture files.

## Prerequisites

- A checkout of the Surgeist repository.
- An installed Rust toolchain with Cargo and support for Rust edition 2024,
  as declared in [Cargo.toml](../Cargo.toml).

The manifest declares no dependencies or optional features. The command below
uses Cargo's offline mode and requires the toolchain to be present already.

## Run and verify

1. Open a terminal in the repository directory containing
   [Cargo.toml](../Cargo.toml).
2. Run the focused test:

   ```sh
   cargo test --offline -p surgeist-test fixtures::tests::layout_ready_metadata_constructor_sets_schema_and_keeps_paths_relative -- --exact
   ```

3. Confirm Cargo reports one passing unit test named
   `fixtures::tests::layout_ready_metadata_constructor_sets_schema_and_keeps_paths_relative`.
   A result with zero executed tests is not the expected proof.

The [test source](../src/fixtures/mod.rs) checks the current schema version,
fixture identity, source and generated artifact paths, generator and adapter
labels, expectation label, and ready status. If Cargo cannot use the installed
toolchain, resolve that prerequisite before interpreting the test result.

Continue with [constructing and annotating fixture records](how-to.md).
