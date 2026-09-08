# Getting started

The first result is a passing test that inserts a keyed semantic node and reads
the same retained identity through a snapshot. This guide is for Rust developers
working in a local checkout of `surgeist-retained`.

## Prerequisites

Use an existing checkout and an installed Rust toolchain with Cargo and support
for the Rust 2024 edition declared in [Cargo.toml](../Cargo.toml). The package has
one library target and no declared external dependencies. It does not start an
application or render a window.

## Run the first test

1. Open a terminal in the directory containing this repository's `Cargo.toml`.
2. Run:

   ```sh
   cargo test --offline -p surgeist-retained tests::canonical_patch_inserts_and_snapshots_children -- --exact
   ```

3. Confirm that `tests::canonical_patch_inserts_and_snapshots_children` ran and
   passed. A successful command that ran zero tests is not the expected result.

The test in [src/tests.rs](../src/tests.rs) creates `Model::empty()`, inserts a
keyed `section` with `Patch::Insert`, and checks the resulting `Report` against
`Snapshot::children` and `Snapshot::get`. The inserted child has the key `main`.

The command uses offline mode and can create local Cargo build artifacts and a
lockfile; those paths are covered by [.gitignore](../.gitignore). If Cargo is
unavailable or the compiler cannot handle the declared edition, use an appropriate
installed toolchain before retrying. If the test fails, retain its diagnostic
output rather than treating the checkout as verified.

## Continue

Read the [how-to guides](how-to.md) to apply atomic mutations, resolve a projected
tree, or turn an event into command records. The [reference](reference.md) maps
those operations to the public source and lists the broader local checks.
