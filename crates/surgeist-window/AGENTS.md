# surgeist-window Crate Guide

Read the [repository guide](../../AGENTS.md) for shared policy and workflow.
This supplement records crate facts and grants no additional authority.

## Product Boundary

This crate owns window, app-host, event-loop, and platform-host contracts: native
identity, events, commands, metrics, handles, and capabilities. Renderers,
surfaces, UI semantics, layout, hit testing, and application behavior remain
outside this crate. The private `winit` adapter stays window-owned.

## Discover The Current Structure

| Fact | Authoritative source |
| --- | --- |
| Package identity, dependencies, features and targets | [Cargo.toml](Cargo.toml) |
| Public front door and reexports | [src/lib.rs](src/lib.rs) |
| Authored requests, commands, and observed state | [src/dsl.rs](src/dsl.rs), [src/command.rs](src/command.rs), [src/descriptor.rs](src/descriptor.rs), [src/geometry.rs](src/geometry.rs) |
| Normalization, planning, callbacks, and lifecycle ownership | [src/normalization.rs](src/normalization.rs), [src/planning.rs](src/planning.rs), [src/pump.rs](src/pump.rs), [src/loop_.rs](src/loop_.rs), [src/registry.rs](src/registry.rs), [src/lease.rs](src/lease.rs) |
| Target capabilities and native lowering | [src/capability.rs](src/capability.rs), [src/winit_adapter.rs](src/winit_adapter.rs), [src/winit_mapping.rs](src/winit_mapping.rs) |
| Focused verification and display-free harness | Tracked test modules in [src/](src/), including [src/tests.rs](src/tests.rs), and [src/testing.rs](src/testing.rs); rustdoc examples in [src/lib.rs](src/lib.rs) and [src/event.rs](src/event.rs) |

[README.md](README.md) links usage, contract and attribution documentation.

## Command Inventory

Select affected checks under the root process/resource policy; commands below
run from this crate directory.

Cover default and `accessibility` when affected. `testing::Runner` exercises
shared lifecycle without a display; `testing::Host` is callback-free. Passing
these harnesses does not establish native behavior on every platform.

```sh
cargo fmt -p surgeist-window -- --check
cargo check -j 1 --offline -p surgeist-window
cargo check -j 1 --offline -p surgeist-window --features accessibility
cargo test -j 1 --offline -p surgeist-window
cargo test -j 1 --offline -p surgeist-window --features accessibility
cargo clippy -j 1 --offline -p surgeist-window --all-targets -- -F unsafe-code -D warnings
cargo clippy -j 1 --offline -p surgeist-window --all-targets --features accessibility -- -F unsafe-code -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc -j 1 --offline -p surgeist-window --no-deps
RUSTDOCFLAGS='-D warnings' cargo doc -j 1 --offline -p surgeist-window --no-deps --features accessibility
```
