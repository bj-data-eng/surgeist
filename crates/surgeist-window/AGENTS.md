# surgeist-window Local Guide

This crate is part of the Surgeist repository and product Cargo workspace.
Read [../../AGENTS.md](../../AGENTS.md) first for workflow, architecture, shared
verification/resource limits, and API generation. This supplement records local
facts; it grants no additional authority. The root guide prohibits executable
`unsafe` in all Surgeist-owned Rust.

This crate owns window, app-host, event-loop, and platform-host contracts: native
identity, events, commands, metrics, handles, and capabilities. Renderers,
surfaces, UI semantics, layout, hit testing, and application behavior remain
outside this crate. The private `winit` adapter stays window-owned.

Consume sibling public front doors. Cross-crate lowering and composition remain
at the root boundary defined by the root guide.

## Sources

[Cargo.toml](Cargo.toml) owns local package/feature facts;
[../../Cargo.toml](../../Cargo.toml) and [../../Cargo.lock](../../Cargo.lock)
own product workspace membership and dependency resolution. Public source remains
authoritative; root-owned API audits are generated through
[../../api/generator](../../api/generator), never hand-edited or copied here.

| Fact | Source |
| --- | --- |
| Package identity, edition, dependencies, features, and targets | `Cargo.toml` |
| Public front door and reexports | `src/lib.rs` |
| Adoption and documentation navigation | `README.md` |
| Usage, contract reference, and architecture explanation | `docs/getting-started.md`, `docs/how-to.md`, `docs/reference.md`, `docs/explanation.md`, and the corresponding `src/` implementation |
| Authored requests, commands, and observed state | `src/dsl.rs`, `src/command.rs`, `src/descriptor.rs`, `src/geometry.rs` |
| Normalization, planning, callbacks, and lifecycle ownership | `src/normalization.rs`, `src/planning.rs`, `src/pump.rs`, `src/loop_.rs`, `src/registry.rs`, `src/lease.rs` |
| Target capabilities and native lowering | `src/capability.rs`, `src/winit_adapter.rs`, `src/winit_mapping.rs` |
| Focused verification and display-free harness | Tracked test modules in `src/`, including `src/tests.rs`, and `src/testing.rs`; rustdoc examples in `src/lib.rs` and `src/event.rs` |
| Project license and third-party attribution coverage | `LICENSE`, `NOTICE.md`, and the upstream texts in `licenses/` |

## Local Verification

Run the applicable commands from this crate directory, one package at a time,
using the root guide's process/resource policy and already-present tooling.
Cargo build/check/test commands may use `--offline --locked` to preserve the
shared product resolution; command inventory is not a requirement to run every
suite for each change.

Cover default and `accessibility` when affected. `testing::Runner` exercises
shared lifecycle without a display; `testing::Host` is callback-free. Passing
these harnesses does not establish native behavior on every platform.

```sh
cargo fmt -p surgeist-window --check
cargo check --offline -p surgeist-window
cargo check --offline -p surgeist-window --features accessibility
cargo test --offline -p surgeist-window
cargo test --offline -p surgeist-window --features accessibility
cargo clippy --offline -p surgeist-window --all-targets -- -F unsafe-code -D warnings
cargo clippy --offline -p surgeist-window --all-targets --features accessibility -- -F unsafe-code -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --offline -p surgeist-window --no-deps
RUSTDOCFLAGS='-D warnings' cargo doc --offline -p surgeist-window --no-deps --features accessibility
```
