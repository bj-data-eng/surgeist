# How-to guides

These procedures assume the pipeline in [getting started](getting-started.md)
is familiar. They use the public exports in [src/lib.rs](../src/lib.rs).

## Register a native element or component

Start with an authored template and the element names your caller intends to
accept.

1. Choose `NativeElementSpec::try_new` for a native name such as `div`, or
   `ComponentSpec::try_new` for a component name such as `Panel`. See the
   [name rules](reference.md#names) for their exact shapes.
2. Supply that element's allowed `AttributeSpec` values. Use an empty vector
   when it accepts no attributes.
3. Build the corresponding `NativeElementRegistry` or `ComponentRegistry` with
   `try_from_specs`, then pass both registries to `validate_template` with the
   parsed document. An unused registry can be empty.
4. Check for `Ok(ValidatedTemplate)`. Invalid specification names or duplicate
   specifications fail during registry construction with `RegistryError`;
   an unregistered authored element fails validation with
   `UnknownNativeElement` or `UnknownComponent`.

The `validates_known_native_and_component_names` and registry rejection cases in
[tests/template_v1.rs](../tests/template_v1.rs) show the expected outcomes.

## Accept the intended attribute forms

Start with an element specification and decide which authored value forms that
element permits.

1. Map each authored form to its [attribute kind](reference.md#attributes):
   for example, `disabled` is `Bool`, `title="Hello"` is `Static`,
   `count={$count}` is `Expression`, and `title="Hello {$user.name}"` is
   `Interpolated`.
2. Use `AttributeRule::one` for one kind, or `AttributeRule::any` for multiple
   kinds. Create an `AttributeSpec` with the attribute's name and rule, and
   include it in the element specification.
3. Validate the document. Confirm that the intended form succeeds and that an
   excluded form produces `ValidationErrorKind::InvalidAttributeValue`.
   An undeclared attribute produces `InvalidAttribute`; repeated authored
   attributes produce `DuplicateAttribute`.

The getting-started example accepts both static and interpolated titles.
`multi_kind_attribute_rule_accepts_interpolated_title` and
`rejects_unknown_attribute_and_invalid_value_kind` in
[tests/template_v1.rs](../tests/template_v1.rs) exercise these distinctions.

## Inspect an authored-template diagnostic

Keep the original source text available when parsing and validating it.

1. If `parse_template` fails, inspect the `ParseError` through `kind()` and
   `span()` to distinguish malformed syntax, names, expressions, and tags.
2. After successful parsing, inspect a failed `validate_template` call through
   the same accessors on `ValidationError` to identify a registry or attribute
   mismatch.
3. Use the span's `start()` and `end()` positions to locate the affected text.
   Each position exposes `line()`, `column()`, and `byte()`; `len_bytes()` on
   the span reports its byte length.
4. Correct the reported input or registry definition and repeat the pipeline.
   Success yields a `ValidatedTemplate` ready for `render_to_rust`.

The error variants are defined in [src/error.rs](../src/error.rs), and multiline
and UTF-8 span examples appear in [tests/template_v1.rs](../tests/template_v1.rs).
