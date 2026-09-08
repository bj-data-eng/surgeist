# Explanation

## Three explicit phases

The template layer separates authored syntax, registry validation, and source
generation. Parsing builds a `TemplateDocument` with typed names, attributes,
expressions, nodes, and source spans. This lets callers inspect authored
structure before deciding which elements their environment supports.

Validation supplies that vocabulary through separate native-element and
component registries. Each specification declares allowed attributes and their
value kinds. A syntactically valid template can therefore fail validation when
an element is unknown or an attribute has an unaccepted form. Successful
validation produces a `ValidatedTemplate`, the input required by
`render_to_rust`. The [public front door](../src/lib.rs) exposes these phase
boundaries; [validate.rs](../src/validate.rs) owns the registry checks.

## Strictness and typed structure

V1 uses familiar element syntax with exact nesting and a limited expression
grammar. It does not recover malformed tags as a browser might. Native and
component names have distinct shapes and types, and attribute values retain
the distinction between boolean, static, expression, and interpolated forms.
That distinction permits a registry to accept an interpolated title while
rejecting it for an attribute that expects another form.

Conditionals and iteration remain nodes with typed expressions and nested
children. Validation visits their branches and bodies without selecting a
branch or running a loop. Source spans survive parsing and validation so
callers can connect structured failures to authored text. See the
[syntax reference](reference.md#v1-syntax) for the supported grammar.

## Symbolic generation and ownership

Rendering currently means code generation. The renderer writes construction
calls under `::surgeist::template` and passes expression source as strings.
For example, `$user.name` remains symbolic, and `$match` does not become a Rust
identifier. The rendering tests assert those strings; they do not establish
that a host implements the emitted calls or evaluates the expressions.

This workspace crate owns strict template and DSL authoring contracts:
parsing, validation, and source generation. The root `surgeist` facade owns
host integration and cross-crate lowering. It also owns the API generator and
generated API audit artifacts; this crate's source remains authoritative for
its implementation. These boundaries are recorded in
[AGENTS.md](../AGENTS.md) and [src/lib.rs](../src/lib.rs).
