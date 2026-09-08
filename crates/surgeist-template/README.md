# surgeist-template

`surgeist-template` is a Rust library for authors of Surgeist templates and the
tools that process them. It parses strict template syntax into typed nodes,
validates elements and attributes against explicit registries, and generates
Rust construction source.

The current V1 surface supports elements, interpolation, conditionals, and
iteration. Expressions remain symbolic; runtime evaluation, host integration,
and cross-crate lowering belong outside this crate.

## Start

From this checkout, run the focused parse → validate → generate test:

```sh
cargo test --offline -p surgeist-template --test template_v1 \
  renders_validated_component_with_expression_attribute_and_interpolation_child -- --exact
```

Expect one passing test. It checks the exact generated construction string,
including an expression attribute and an interpolated child. See
[getting started](docs/getting-started.md) for prerequisites and the Rust example.

## Documentation

- [Getting started](docs/getting-started.md): verify the pipeline and follow a complete example.
- [How-to guides](docs/how-to.md): register elements, permit attribute forms, and handle diagnostics.
- [Reference](docs/reference.md): V1 syntax, public interfaces, and source locations.
- [Explanation](docs/explanation.md): phase boundaries, strictness, and symbolic generation.

## License and attribution

The project uses the [MIT license](LICENSE). See [third-party attribution](NOTICE.md)
for the scope and result of the dependency and bundled-material inventory.
