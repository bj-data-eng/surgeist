# surgeist-shape

Resolved geometry and shape primitives for developers building Surgeist
rendering and layout-adjacent surfaces. The Rust library provides validated
geometry, paths, bounds, containment, transforms, geometry keys, strokes, and
dash geometry through one [public API](src/lib.rs).

The crate is version `0.1.0`. It owns geometric contracts; style resolution,
layout, and rendering belong to its consumers. Arbitrary-path inflation,
dashing, and inside/outside stroke bounds are currently unsupported.

## Start

From a prepared checkout, run the existing public-API smoke test:

```sh
cargo test --offline --locked -p surgeist-shape --lib tests::public_front_doors_construct_valid_shape_and_dash_geometry -- --exact
```

It constructs a rounded rectangle and a dashed stroke, then verifies that the
result contains dash geometry. Expect `1 passed; 0 failed`.
See [Getting started](docs/getting-started.md) for prerequisites and the complete
first-success walkthrough.

## Documentation

| Guide | Purpose |
| --- | --- |
| [Getting started](docs/getting-started.md) | Prepare a checkout and verify the public API. |
| [How-to](docs/how-to.md) | Construct shapes and paths, query bounds, and generate dashes. |
| [Reference](docs/reference.md) | Find API groups, package facts, and capability limits. |
| [Explanation](docs/explanation.md) | Understand validation, geometry identity, and ownership boundaries. |

## License and attribution

This project is licensed under the [MIT License](LICENSE).
[Third-party attribution](NOTICE.md) includes the corresponding dependency
license texts.
