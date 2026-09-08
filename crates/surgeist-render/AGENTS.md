# surgeist-render Local Guide

This crate is part of the Surgeist repository and product Cargo workspace.
Read [../../AGENTS.md](../../AGENTS.md) first for workflow, architecture, shared
verification/resource limits, and API generation. This supplement records local
facts; it grants no additional authority. The root guide prohibits executable
`unsafe` in all Surgeist-owned Rust.

This crate owns scene validation and normalization, GPU execution, surface resources,
explicit headless readback, and diagnostics. Callers own layout, shaping, and
style interpretation; hosts own application/window/browser lifecycle. Current
public authored style models and normalization remain existing API facts, not a
new cross-crate ownership rule. Production rendering is GPU-only; CPU reference
algorithms are test-only. Pixel publication and successful statistics are atomic;
CPU-visible headless pixels require explicit readback.

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
| Public front door and exported contracts | `src/lib.rs` and the reexported definitions |
| Human documentation and entry example | `README.md` and `docs/` |
| Authored commands and normalization | `src/scene.rs`, `src/command.rs`, and `src/style/` |
| Planning, execution, and publication | `src/frame/`, `src/renderer/`, `src/pass/`, `src/gpu_transaction/`, and `src/backend/` |
| GPU resources, shaders, and internal raster engine | `src/resource/`, `src/shader/`, `src/shaders/`, and `src/vello_engine/` |
| Surface lifecycle and explicit readback | `src/surface.rs` and `src/readback/` |
| Semantic and runtime capabilities, errors, and statistics | `src/capability.rs`, `src/error.rs`, and `src/stats.rs` |
| Focused verification | `src/tests/`, tracked local `#[cfg(test)]` modules, `src/reference/`, and `tests/fixtures/` |
| Live presented smoke target | `examples/render_window_smoke.rs` and its `Cargo.toml` target |
| Project license and third-party attribution | `LICENSE`, `NOTICE.md`, `NOTICE-VELLO.md`, `LICENSES/`, and `tests/fixtures/fonts/ahem/` |

## Local Verification

Run the applicable commands from this crate directory, one package at a time,
using the root guide's process/resource policy and already-present tooling.
Cargo build/check/test commands may use `--offline --locked` to preserve the
shared product resolution; command inventory is not a requirement to run every
suite for each change.

Native feature combinations are default, `render-window`, `render-web`, and
`render-window,render-web`. Native tests include real GPU execution and require
a compatible adapter; serial test threads do not bound GPU allocation. Select
`tests::platform::real_gpu_smoke_emits_no_uncaptured_error` for the initial 2×2
smoke rather than assuming the full GPU suite fits the host. Wasm coverage is
compile-only; browser execution/canvas presentation needs root integration
evidence. The native smoke example requires a graphical session and exits after
one direct and one GPU-graph frame.

```sh
CARGO_NET_OFFLINE=true cargo check -p surgeist-render
CARGO_NET_OFFLINE=true cargo test -p surgeist-render
CARGO_NET_OFFLINE=true cargo clippy -p surgeist-render --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-render --check
```

```sh
CARGO_NET_OFFLINE=true cargo check -p surgeist-render --target wasm32-unknown-unknown --features render-web --lib --tests
```

```sh
CARGO_NET_OFFLINE=true cargo run -p surgeist-render --example render_window_smoke --features render-window
CARGO_NET_OFFLINE=true cargo run -p surgeist-render --example render_window_smoke --features render-window,render-web
```
