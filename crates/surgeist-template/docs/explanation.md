# Explanation

## Authored structure, validation and outputs

The template layer separates authored syntax, registry validation, and source
generation. Parsing builds a `TemplateDocument` with typed names, attributes,
expressions, nodes, and source spans. This lets callers inspect authored
structure before deciding which elements their environment supports.

Validation supplies that vocabulary through separate native-element and
component registries. Each specification declares allowed attributes and their
value kinds. A syntactically valid template can therefore fail validation when
an element is unknown or an attribute has an unaccepted form. Successful
validation produces a `ValidatedTemplate`, the input required by
`render_to_rust` or, for concrete native values, `append_static_html`.
The [public front door](../src/lib.rs) exposes these phase
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

Rendering means code generation. The renderer emits one Rust expression using
real public constructors under `::surgeist::template`. Its value is an authored
`TemplateDocument`, preserving typed expressions, native/component names, ordered
attributes and children, control flow and diagnostic spans. `$user.name` remains
a typed variable path, and `$match` does not become a Rust identifier. Floating
point literals retain their exact bits, including negative zero and NaN payloads.
The root compiled-consumer test executes generated Rust and revalidates its result.

Native callers can also construct the same AST. Checked positions require
one-based line/column coordinates; spans reject reversed ranges and inconsistent
zero-length ranges. Coordinates are diagnostic metadata without a source-buffer
identity. An `if` node requires its initial branch. Name/path construction checks
intrinsic syntax, while unknown elements/attributes and unsupported value forms
remain the registry validator's responsibility. Native construction and generated
output cannot construct a `ValidatedTemplate` directly.

This workspace crate owns strict template and DSL authoring contracts:
parsing, native AST construction, validation and source generation. Template owns
its direct HTML/DOM adapters. Runtime
evaluation and host policy remain external to this crate. Repository root owns
workspace wiring, the `surgeist` facade and integration verification, including
the compiled consumer, and the API generator and generated audit artifacts.
This crate's source remains authoritative for its implementation. These boundaries are recorded in
[AGENTS.md](../AGENTS.md) and [src/lib.rs](../src/lib.rs).

## HTML vocabulary and concrete DOM composition

The shared HTML inventory supplies a closed known-name registry profile. Template
unions global and contextual attribute rows rather than copying tag/attribute
tables or selecting a spelling's first contextual row. The caller still selects
template value forms with `AttributeRule`. The inventory is discovery evidence;
it does not prove complete HTML applicability/value conformance, supply defaults
or cover open attribute families and custom/foreign names.

For concrete validated nodes, `append_static_html` prepares an owned iterative
plan and then queues ordinary edits into the caller's `DomBatch`. It retains exact
text and ordered raw attributes, creates Elements in the actual HTML namespace,
and returns batch-local root tickets. Existing nodes and same-batch document /
parent tickets use the same DOM operand types. The caller decides when to apply
the complete batch. DOM owns actual native-family/identity/capacity admission and
atomic publication; Template neither becomes a live tree owner nor fabricates
HTML document metadata.

Symbolic values return a typed unresolved error with their authored span before
any edits are queued. HTML template contents require a canonical contents target
and are explicitly unsupported by this adapter. Other native families are
admitted or rejected by the real DOM creation operation at apply time. Immutable
HTML installation and dynamic evaluation remain separate work; this adapter does
not reparse source, reset imported native state or evaluate a new DSL.
