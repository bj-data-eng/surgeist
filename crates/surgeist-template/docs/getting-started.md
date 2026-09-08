# Getting started

This guide follows the public parse → validate → generate pipeline. The result
is a Rust construction string with symbolic expressions; it does not execute a
template.

## Prerequisites

Use a checkout of the Surgeist repository and an already installed Rust toolchain with
Cargo that can compile it. [Cargo.toml](../Cargo.toml) declares edition 2024 and
does not declare a minimum Rust version. The focused test runs offline and
does not need a Surgeist host.

## Verify the pipeline

1. Open a shell at the crate directory, beside [Cargo.toml](../Cargo.toml).
2. Run the existing integration test:

   ```sh
   cargo test --offline -p surgeist-template --test template_v1 \
     renders_validated_component_with_expression_attribute_and_interpolation_child -- --exact
   ```

3. Check that exactly one test ran and passed. The test in
   [tests/template_v1.rs](../tests/template_v1.rs) parses a `Panel`, validates its
   expression attribute, and compares the complete generated string with the
   expected construction calls. A pass establishes that crate-local pipeline;
   the generated string is not compiled or evaluated by this test.

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

assert!(rust_source.contains(r#"::surgeist::template::component("Panel""#));
```

`parse_template` produces the typed document before registry lookup. Both
`div` and `Panel` then need registered specifications. The title rule accepts
static or interpolated values, while `visible` accepts an expression.
`validate_template` returns the `ValidatedTemplate` required by
`render_to_rust`; the final assertion checks that generation includes the
component construction call.

Continue with the [how-to guides](how-to.md) to adapt registries and inspect
failures, or consult the [syntax and API reference](reference.md).
