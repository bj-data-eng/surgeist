# Getting started

Run one focused test to construct a rounded rectangle, attach a dashed stroke,
and verify that it produces nonempty dash geometry. This proves a public API
path through the library; it does not open a window or render an image.

## Prerequisites

- A checkout of the Surgeist repository.
- Installed Rust and Cargo supporting the Rust 2024 edition declared in
  [Cargo.toml](../Cargo.toml). The manifest does not declare a minimum Rust version.
- The shared [product Cargo.lock](../../../Cargo.lock) and its dependencies
  in Cargo's cache.

The command below uses the existing dependency resolution without fetching
packages or changing the lockfile. A fresh checkout without that local setup
may fail its prerequisite check before running the test.

## First success

1. Open a terminal in the crate directory, the directory containing
   [Cargo.toml](../Cargo.toml) and [src/lib.rs](../src/lib.rs).
2. Run the same focused proof used by the [README](../README.md):

   ```sh
   cargo test --offline --locked -p surgeist-shape --lib tests::public_front_doors_construct_valid_shape_and_dash_geometry -- --exact
   ```

3. Verify that Cargo reports the named test as `ok` and a result of `1 passed`
   with `0 failed`. If Cargo cannot resolve a cached dependency or needs to
   update the lockfile, the prerequisite setup is incomplete; that output does
   not establish a geometry failure.

The [test body](../src/tests.rs) creates an 80-by-40 rectangle, assigns corner
radii of 8, and resolves a centered stroke of width 2 with dashed geometry.
Its final assertion checks `!geometry.is_empty()`.

## Continue

Use the [how-to guide](how-to.md) to construct shapes, build paths, inspect
bounds, and handle typed errors in your own caller. The
[reference](reference.md) maps the public exports to their source files.
