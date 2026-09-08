# Getting started

Run one existing test through the public `compute_leaf` boundary and see how
measured content becomes a layout box. This guide is for a local checkout of
`surgeist-layout`; it does not require a browser or the generator feature.

## Prerequisites

- The Surgeist checkout containing this [crate manifest](../Cargo.toml) and
  the shared [product Cargo.lock](../../../Cargo.lock).
- Rust and Cargo satisfying the manifest's Rust `1.97` requirement and 2024 edition.
- An existing Cargo cache sufficient to resolve the lockfile and build the
  default library tests, including their development dependencies.

The command below uses the lockfile and runs offline. An unavailable toolchain or
uncached dependency means these prerequisites are incomplete; it does not
establish a result for the layout test.

## Run the calculation

1. Open a terminal at the crate directory, beside `Cargo.toml`.
2. Run the exact library test:

   ```sh
   cargo test --locked --offline -p surgeist-layout --lib leaf_tests::leaf_layout_adds_padding_and_border_to_measured_outer_size -- --exact
   ```

3. Check that Cargo reports the named test as `ok`, with `1 passed` and `0 failed`.
   Other library tests are filtered out; zero executed tests is not this result.

The [test implementation](../src/leaf_tests.rs) is
`leaf_layout_adds_padding_and_border_to_measured_outer_size`. It calls
[`compute_leaf`](../src/measurement.rs) with automatic dimensions, a provider
that measures 40 × 12, padding of 3 on each edge, and a border of 2 on each edge.
The assertions verify a final size of 50 × 22 and `content_size` of 46 × 18.
The function is exported from the [public crate root](../src/lib.rs).

Continue with [how to run broader checks or select scalar precision](how-to.md),
or use the [API reference](reference.md) to find the tree-layout front door.
