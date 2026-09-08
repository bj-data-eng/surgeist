# surgeist-layout

A Rust library for developers integrating layout into Surgeist document surfaces.
It computes box, line, and scroll geometry from layout-ready inputs, with block,
inline, flex, grid, subgrid, and experimental grid-lanes algorithms.

The crate is at version `0.1.0`. Its grid support is a defined subset, and it does
not provide authored CSS parsing, text shaping, rendering, or a live scrolling
runtime. See the [scope and architecture](docs/explanation.md) for the boundaries.

## Try a layout calculation

From this checkout, run one focused leaf-layout test:

```sh
cargo test --locked --offline -p surgeist-layout --lib leaf_tests::leaf_layout_adds_padding_and_border_to_measured_outer_size -- --exact
```

Expected: one passing test. It verifies that a measured 40 × 12 box with padding
and borders produces 50 × 22 layout output. Prerequisites and the test walkthrough
are in [getting started](docs/getting-started.md).

## Documentation

- [Getting started](docs/getting-started.md): run and understand the first calculation.
- [How-to](docs/how-to.md): choose scalar precision and run layout or corpus checks.
- [Reference](docs/reference.md): public APIs, value contracts, features, and commands.
- [Explanation](docs/explanation.md): ownership, grid scope, and physical geometry.

## License and attribution

[MIT license](LICENSE) · [Third-party notices](NOTICE.md)
