# Getting Started

This guide prepares the Surgeist checkout and verifies its library entry point.
A successful run compiles the facade and its default production dependencies,
then passes the root identity test.

## Prerequisites

- Git.
- Rust 1.97 or newer with Cargo, matching the compatibility floor in
  [Cargo.toml](../Cargo.toml).
- Dependencies selected by the committed [Cargo.lock](../Cargo.lock) available
  in the local Cargo cache. Verification uses offline mode and does not retrieve
  missing packages.

Toolchain installation and initial dependency retrieval are environment setup;
the offline check assumes they are complete. Cargo reports the missing package
or incompatible toolchain when that prerequisite is unmet. The separate API and
optional audit tools have additional prerequisites described in their sources.

## Prepare The Checkout

1. Clone the repository:

   ```sh
   git clone https://github.com/bj-data-eng/surgeist.git
   cd surgeist
   ```

   All 15 crates under `crates/` are ordinary tracked source directories in this
   repository. No separate crate checkout initialization is required.

2. From the root directory, run the serial library check:

   ```sh
   cargo test --offline --locked -j 1 -p surgeist --lib -- --test-threads=1
   ```

## Observe First Success

The test output includes `tests::exposes_crate_identity ... ok` and one passing
test. The [implementation](../src/lib.rs) checks that
`surgeist::crate_name()` returns `"surgeist"`.

This verifies the current facade entry point. The workspace has 16 product
members, but its default member is only root. The command above does not run
the other members' tests, a root example application, or a development harness.
Continue with the [how-to guide](how-to.md) to select an affected package or API
audit without starting a broad workspace test run.
