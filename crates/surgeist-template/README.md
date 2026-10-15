# surgeist-template

`surgeist-template` is a Rust library for authors of Surgeist templates and the
tools that process them. It parses strict template syntax into typed nodes,
validates elements and attributes against explicit registries, and generates
Rust source that constructs the same authored AST through public constructors.

The current V1 surface supports elements, interpolation, conditionals, and
iteration. Expressions remain symbolic; runtime evaluation and host policy belong
to their consuming owners. Template owns its direct HTML/DOM integration as those
separate adapters are contributed.

## Start

From the repository root, run the compiled generated-output consumer:

```sh
pisct run --timeout-seconds 900 -- cargo test --offline --locked -j 1 \
  -p surgeist --test template_generated_rust
```

Expect one passing test. It compiles and executes the actual generated Rust,
checks literal symbolic values and spans, and revalidates the authored result. See
[getting started](docs/getting-started.md) for prerequisites and the Rust example.

## Documentation

- [Getting started](docs/getting-started.md): verify the pipeline and follow a complete example.
- [How-to guides](docs/how-to.md): register elements, permit attribute forms, and handle diagnostics.
- [Reference](docs/reference.md): V1 syntax, public interfaces, and source locations.
- [Explanation](docs/explanation.md): phase boundaries, strictness, and symbolic generation.

## License and attribution

The project uses the [MIT license](LICENSE). See [third-party attribution](NOTICE.md)
for the scope and result of the dependency and bundled-material inventory.
