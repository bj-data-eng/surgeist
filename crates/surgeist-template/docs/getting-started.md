# Getting started

This guide follows the public parse → validate → generate pipeline. The result
is Rust source that constructs an authored AST with symbolic expressions; it
does not evaluate a template.

## Prerequisites

Use a checkout of the Surgeist repository and an already installed Rust toolchain with
Cargo that can compile it. [Cargo.toml](../Cargo.toml) declares edition 2024 and
does not declare a minimum Rust version. The focused test runs offline and
does not need a Surgeist host.

## Verify the pipeline

1. Open a shell at the repository root.
2. Run the existing integration test:

   ```sh
   pisct run --timeout-seconds 900 -- cargo test --offline --locked -j 1 \
     -p surgeist --test template_generated_rust
   ```

3. Check that exactly one test ran and passed. The test in
   [root compiled consumer](../../../tests/template_generated_rust.rs) parses a
   `Panel`, validates its symbolic content, and compiles and executes the exact
   renderer output. It asserts literal values and spans, then revalidates the
   constructed authored AST through explicit registries.

## Follow a complete example

The following Rust example uses only the crate's
[public exports](../src/lib.rs). The template has a component, a native child,
an interpolated title, an expression attribute, and interpolated child text.

```rust
use surgeist_template::{
    AttributeKind, AttributeRule, AttributeSpec, ComponentRegistry, ComponentSpec,
    NativeElementRegistry, NativeElementSpec, parse_template, render_to_rust,
    validate_template,
};

let document = parse_template(
    r#"<Panel title="Hello {$user.name}" visible={$visible}><div>{$message}</div></Panel>"#,
)
.expect("template parses");

let native = NativeElementRegistry::try_from_specs(vec![
    NativeElementSpec::try_new("div", Vec::new()).expect("native spec"),
])
.expect("native registry");

let components = ComponentRegistry::try_from_specs(vec![
    ComponentSpec::try_new(
        "Panel",
        vec![
            AttributeSpec::try_new(
                "title",
                AttributeRule::any(
                    AttributeKind::Static,
                    [AttributeKind::Interpolated],
                ),
            )
            .expect("title attr"),
            AttributeSpec::try_new("visible", AttributeRule::one(AttributeKind::Expression))
                .expect("visible attr"),
        ],
    )
    .expect("component spec"),
])
.expect("component registry");

let validated = validate_template(&document, &native, &components).expect("template validates");
let rust_source = render_to_rust(&validated);

println!("{rust_source}");
```

`parse_template` produces the typed document before registry lookup. Both
`div` and `Panel` then need registered specifications. The title rule accepts
static or interpolated values, while `visible` accepts an expression.
`validate_template` returns the `ValidatedTemplate` required by
`render_to_rust`. Compile the returned expression in a consumer that depends on
the `surgeist` facade; its value is a `surgeist::template::TemplateDocument`.
Pass that value through `validate_template` with the consumer's registries before
further lowering. The generated code performs no runtime expression evaluation.

Continue with the [how-to guides](how-to.md) to adapt registries and inspect
failures, or consult the [syntax and API reference](reference.md).
