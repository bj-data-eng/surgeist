# surgeist-template

`surgeist-template` is a Rust library for authors of Surgeist templates and the
tools that process them. It parses strict template syntax into typed nodes,
validates elements and attributes against explicit registries, and generates
Rust source that constructs the same authored AST through public constructors.

The current V1 surface supports elements, interpolation, conditionals, and
iteration. Expressions remain symbolic; runtime evaluation and host policy belong
to their consuming owners. Template directly consumes HTML's known-name metadata
and queues concrete HTML construction through the canonical DOM batch API.

`NativeElementRegistry::from_html_metadata` unions global and contextual shared
attribute records, with an explicit `AttributeRule` for template value forms.
This finite discovery profile does not certify full HTML conformance or cover
custom/foreign names and open attribute families; manual registries remain
available. `append_static_html` accepts a `ValidatedTemplate`, a caller-owned
`DomBatch`, and document/parent `NodeOperand`s, including same-batch tickets.
It preserves ordered attributes/text and returns root creation tickets for the
caller's successful commit. Symbolic values return span-bearing unresolved
errors before queuing; HTML template contents return an explicit unsupported
target error. DOM admits native families and limits when applying the composed
batch. HTML installation and dynamic evaluation are separate contributions.

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
