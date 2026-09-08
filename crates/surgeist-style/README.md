# surgeist-style

A Rust library for Surgeist's typed style model, cascade, and resolution. It lets
host integrations construct style declarations, match rules against a supplied
tree, and obtain resolved visual, layout, and text property values with
diagnostics and invalidation facts.

The crate is version 0.1.0 and implements a subset of CSS-style contracts. It does
not parse CSS text or execute layout, text shaping, rendering, or animation
timelines. The [architecture guide](docs/explanation.md) explains the boundary
and links the existing correctness and completeness review.

## Start

From a checkout with Rust and the dependencies already available, run:

```sh
cargo test --offline -p surgeist-style --lib resolver::tests::resolved_edge_getters_assemble_side_longhands -- --exact
```

Expect one passing test. It constructs typed declarations and verifies resolved
margin, padding, border, and inset values. See [Getting started](docs/getting-started.md)
for prerequisites and the path from this test to using the public API.

## Documentation

| Guide | Purpose |
| --- | --- |
| [Getting started](docs/getting-started.md) | Run the first resolution example and find public construction examples. |
| [How-to](docs/how-to.md) | Construct declarations, resolve styles, inspect diagnostics, and handle changes. |
| [Reference](docs/reference.md) | Find interfaces, source modules, compatibility facts, and verification commands. |
| [Explanation](docs/explanation.md) | Understand style phases, host responsibilities, caching, and limitations. |

## License

`surgeist-style` is licensed under the [MIT License](LICENSE). See
[third-party attribution](NOTICE.md) for dependency coverage and upstream license
material.
