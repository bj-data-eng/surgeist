# surgeist-template Crate Guide

Read the [repository guide](../../AGENTS.md) for shared policy and workflow.
This supplement records crate facts and grants no additional authority.

## Product Boundary

This crate owns strict template parsing, registry-driven validation, and Rust source
generation and native authored AST construction. Expressions remain symbolic.
Runtime evaluation and host policy remain outside this crate. `render_to_rust`
in `src/render.rs` produces a Rust expression constructing `TemplateDocument`
from a `ValidatedTemplate`; the authored result requires registry validation.

Consume sibling public front doors directly. Template owns its HTML/DOM adapters
as those contributions are implemented. Root owns workspace wiring, the facade,
integration verification and generated API artifacts; it is not a mandatory
lowering path. `NativeElementRegistry::from_html_metadata` consumes the shared
known-name inventory with explicit template value-form policy. This discovery
profile is not complete HTML conformance, defaults or open-family recognition.
`append_static_html` preflights concrete values before queuing ordinary DOM edits;
the caller applies the batch, and DOM owns actual native-family/identity/limit
admission. Keep HTML template contents an explicit unsupported target until its
canonical association is available. No expression evaluation is supplied.

## Discover The Current Structure

| Fact | Authoritative source |
| --- | --- |
| Package identity, dependencies, features and targets | [Cargo.toml](Cargo.toml) |
| Public front door and reexports | [src/lib.rs](src/lib.rs) |
| Template syntax and authored structure | [src/parser.rs](src/parser.rs), [src/lexer.rs](src/lexer.rs), and [src/ast.rs](src/ast.rs) |
| Expression syntax and name constraints | [src/expr.rs](src/expr.rs) and [src/name.rs](src/name.rs) |
| Registry contracts and validated structure | [src/validate.rs](src/validate.rs) |
| Rust source generation | [src/render.rs](src/render.rs) |
| HTML inventory registry and static DOM composition | [src/validate.rs](src/validate.rs), [src/dom.rs](src/dom.rs), [tests/html_dom.rs](tests/html_dom.rs) |
| Diagnostic kinds and source positions | [src/error.rs](src/error.rs) and [src/span.rs](src/span.rs) |
| Focused verification | Tracked `#[cfg(test)]` modules in [src/](src/) and [tests/template_v1.rs](tests/template_v1.rs) |

[README.md](README.md) links usage, contract and attribution documentation.

## Command Inventory

Select affected checks under the root process/resource policy; commands below
run from this crate directory.

Verification includes source-module tests, `tests/template_v1.rs` and
`tests/native_ast.rs`, `tests/html_dom.rs` and the static adapter's doc example.
The root `tests/template_generated_rust.rs` compiles and
executes actual generated Rust against the real facade. Keep generated Rust safe
and validated-template expressions symbolic.

```sh
cargo check -j 1 -p surgeist-template
cargo test -j 1 -p surgeist-template
cargo clippy -j 1 -p surgeist-template --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-template -- --check
```
