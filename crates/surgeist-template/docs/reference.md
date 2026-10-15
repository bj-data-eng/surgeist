# Reference

## Package and public entry point

[Cargo.toml](../Cargo.toml) defines package `surgeist-template` version `0.1.0`,
edition 2024, and library import name `surgeist_template`. It declares no
dependencies, features, or `rust-version` requirement.
[src/lib.rs](../src/lib.rs) is the public front door; implementation modules
are private and selected types and functions are reexported there.

| Interface | Input and result | Source |
| --- | --- | --- |
| `parse_template` | `&str` → `Result<TemplateDocument, ParseError>` | [parser.rs](../src/parser.rs) |
| `validate_template` | Document, native registry, and component registry references → `Result<ValidatedTemplate, ValidationError>` | [validate.rs](../src/validate.rs) |
| `render_to_rust` | `&ValidatedTemplate` → `String` | [render.rs](../src/render.rs) |

`TemplateDocument::nodes()` exposes parsed `Node` values: `Element`, `Text`,
`Interpolation`, `If`, and `ForEach`. `ValidatedTemplate::nodes()` preserves
these structures while distinguishing `NativeElement` and `ComponentElement`
in `ValidatedNode`. Element, attribute, text, interpolation, and control-flow
nodes carry source spans. See [ast.rs](../src/ast.rs),
[validate.rs](../src/validate.rs), and [span.rs](../src/span.rs).

## V1 syntax

Templates use HTML-like elements with exact open/close matching, self-closing
elements, text nodes, and `{* ... *}` comments. The parser does not perform
browser-style recovery. Text interpolation uses `{$expr}`, as in
`Hello {$user.name}`. See [parser.rs](../src/parser.rs) and the
[integration cases](../tests/template_v1.rs).

### Names

All character classes below are ASCII. Both native and component names must
also be registered for validation to succeed. The exact predicates live in
[name.rs](../src/name.rs).

| Name | First character | Remaining characters | Example |
| --- | --- | --- | --- |
| Native element | Lowercase letter | Lowercase letters, digits, `_`, `-` | `custom-element` |
| Component | Uppercase letter | Letters, digits, `_` | `Panel`, `UI_Panel2` |
| Attribute | Letter or `_` | Letters, digits, `_`, `-` | `data-id` |
| Variable or path field | Letter or `_` | Letters, digits, `_` | `_item2` |

Component names require an uppercase first character, without imposing a
particular casing convention on the rest of the name.

### Attributes

| Authored form | Parsed kind | Example |
| --- | --- | --- |
| Bare attribute | `Bool` | `disabled` |
| Static quoted or unquoted value | `Static` | `title="Hello"`, `data-id=main` |
| Unquoted braced expression | `Expression` | `enabled={$is_enabled}`, `count={42}` |
| Quoted value containing interpolation | `Interpolated` | `title="Hello {$user.name}"` |

`NativeElementRegistry` and `ComponentRegistry` declare known elements and their
allowed attributes. `AttributeRule::one` accepts one `AttributeKind`;
`AttributeRule::any` accepts a nonempty selection. Constructors reject invalid
names and duplicate registry or attribute specifications. Validation rejects
unknown elements, unknown attributes, duplicate authored attributes, and value
kinds outside the declared rule. See [validate.rs](../src/validate.rs).

### Control flow

Supported control flow is limited to conditional and iteration blocks:

```text
{if $visible}...{elseif $fallback}...{else}...{/if}
{foreach $items as $item}...{foreachelse}...{/foreach}
```

The alternative branches are optional. `foreach` requires one `as` clause and
a variable binding prefixed by `$`. Branches and loop bodies retain nested
template nodes. The [parser](../src/parser.rs) rejects stray closing tags and
unsupported template tags.

### Expressions

The symbolic expression subset includes:

- Scalar literals: booleans, `null`, integers, decimal floating-point numbers,
  and strings.
- Variable paths such as `$user.name` and `$items[0].name`. Indexes are
  nonnegative integers; adjacent index segments such as `$items[0][1]` are
  rejected.
- Parentheses; unary `!` and `-`; arithmetic `+`, `-`, `*`, `/`, `%`;
  comparisons `==`, `!=`, `>`, `>=`, `<`, `<=`; and boolean `&&`, `||`.

Function and method calls, mutation, array literals, ternary and null-coalescing
operators, includes, inheritance, and arbitrary template blocks inside start
tags are outside V1. The expression parser also rejects the variable roots
`GLOBALS`, `_REQUEST`, `_GET`, `_POST`, `_COOKIE`, `_SERVER`, `_SESSION`, `_ENV`,
and `_FILES`. See [expr.rs](../src/expr.rs) and [parser.rs](../src/parser.rs).

## Generated source

`render_to_rust` emits one complete Rust expression with result type
`surgeist::template::TemplateDocument`. It uses actual public AST constructors
through the facade, with fully qualified names. Typed expressions remain symbolic;
no expression evaluation or source reparse occurs. Spans retain their authored
coordinates, rather than identifying the generated Rust file. Float literals use
`f64::from_bits` so all bit patterns survive. Callers validate the constructed
document against their actual registries before obtaining `ValidatedTemplate`.
The root compiled-consumer test exercises [render.rs](../src/render.rs) output.

## Native authored construction

`TemplateDocument::from_nodes` and the `Node` constructors build the same authored
model as parsing. `Node::if_block` returns `AstConstructionError::EmptyIfBranches`
when no initial branch is supplied. Checked names, paths and attributes use their
existing constructors; registry validation remains separate.

`SourcePos::try_new` requires nonzero line and column, with a zero-based byte
offset. `SourceSpan::try_new` requires ordered byte and line/column endpoints and
identical coordinates for a zero-byte range. `SpanError` reports the violated
condition. These are caller-supplied diagnostic coordinates, not a source-buffer
identity or proof of membership or containment.

## Diagnostics and verification sources

`ParseError` and `ValidationError` expose `kind()` and `span()`. `RegistryError`
reports specification-construction failures. The variants live in
[error.rs](../src/error.rs) and [validate.rs](../src/validate.rs).

Crate-local verification consists of unit tests in [src/](../src/) and the
integration suite [tests/template_v1.rs](../tests/template_v1.rs).
[AGENTS.md](../AGENTS.md#command-inventory) owns the repository command inventory.
