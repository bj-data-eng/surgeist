# surgeist-template Local Guide

This crate is part of the Surgeist repository and product Cargo workspace.
Read [../../AGENTS.md](../../AGENTS.md) first for workflow, architecture, shared
verification/resource limits, and API generation. This supplement records local
facts; it grants no additional authority. The root guide prohibits executable
`unsafe` in all Surgeist-owned Rust.

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

## Sources

[Cargo.toml](Cargo.toml) owns local package/feature facts;
[../../Cargo.toml](../../Cargo.toml) and [../../Cargo.lock](../../Cargo.lock)
own product workspace membership and dependency resolution. Public source remains
authoritative; root-owned API audits are generated through
[../../api/generator](../../api/generator), never hand-edited or copied here.

| Fact | Source |
| --- | --- |
| Package identity, version, edition, dependencies, features, and targets | `Cargo.toml` |
| Public front door and reexports | `src/lib.rs` |
| Template syntax and authored structure | `src/parser.rs`, `src/lexer.rs`, and `src/ast.rs` |
| Expression syntax and name constraints | `src/expr.rs` and `src/name.rs` |
| Registry contracts and validated structure | `src/validate.rs` |
| Rust source generation | `src/render.rs` |
| HTML inventory registry and static DOM composition | `src/validate.rs`, `src/dom.rs`, `tests/html_dom.rs` |
| Diagnostic kinds and source positions | `src/error.rs` and `src/span.rs` |
| Human documentation and navigation | `README.md` and its linked guides in `docs/` |
| Focused verification | Tracked `#[cfg(test)]` modules in `src/` and `tests/template_v1.rs` |
| Project licensing and third-party attribution | `LICENSE`, the license field in `Cargo.toml`, and `NOTICE.md` |

## Local Verification

Run the applicable commands from this crate directory, one package at a time,
using the root guide's process/resource policy and already-present tooling.
Cargo build/check/test commands may use `--offline --locked` to preserve the
shared product resolution; command inventory is not a requirement to run every
suite for each change.

Verification includes source-module tests, `tests/template_v1.rs` and
`tests/native_ast.rs`, `tests/html_dom.rs` and the static adapter's doc example.
The root `tests/template_generated_rust.rs` compiles and
executes actual generated Rust against the real facade. Keep generated Rust safe
and validated-template expressions symbolic.

```sh
cargo check -p surgeist-template
cargo test -p surgeist-template
cargo clippy -p surgeist-template --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-template --check
```
