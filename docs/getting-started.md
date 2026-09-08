# Getting Started

This guide prepares the root Surgeist checkout and verifies its library entry
point. A successful run compiles the facade and its default production
dependencies, then passes the root identity test.

## Prerequisites

- Git with submodule support.
- Rust 1.97 or newer with Cargo, matching the compatibility floor in
  [Cargo.toml](../Cargo.toml).
- The selected dependencies available in the local Cargo cache. The verification
  command below uses offline mode and does not retrieve missing packages.

Toolchain installation and initial dependency retrieval are environment setup;
the offline check assumes they are complete. Cargo reports the missing package
or incompatible toolchain when that prerequisite is unmet.

## Prepare The Checkout

1. Clone the repository and its pinned submodules:

   ```sh
   git clone --recurse-submodules https://github.com/bj-data-eng/surgeist.git
   cd surgeist
   ```

2. For an existing clone with uninitialized submodules, populate the revisions
   selected by root:

   ```sh
   git submodule update --init --recursive
   ```

   The authoritative repository URLs are in [.gitmodules](../.gitmodules).
   Each leaf remains an independently owned Git repository.

3. From the root directory, run the library check:

   ```sh
   cargo test --offline -p surgeist --lib
   ```

## Observe First Success

The test output includes `tests::exposes_crate_identity ... ok` and one passing
test. The [test implementation](../src/lib.rs) checks that
`surgeist::crate_name()` returns `"surgeist"`.

This verifies the current facade entry point. The baseline has no root example
application or native development harness, and this command does not exercise
the leaf repositories' own tests. Continue with the
[how-to guide](how-to.md) for feature and API-audit tasks.
