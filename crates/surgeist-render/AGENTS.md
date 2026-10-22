# surgeist-render Crate Guide

Read the [repository guide](../../AGENTS.md) for shared policy and workflow.
This supplement records crate facts and grants no additional authority.

## Product Boundary

This crate owns scene validation and normalization, GPU execution, surface resources,
explicit headless readback, and diagnostics. Callers own layout, shaping, and
style interpretation; hosts own application/window/browser lifecycle. Current
public authored style models and normalization remain existing API facts, not a
new cross-crate ownership rule. Production rendering is GPU-only; CPU reference
algorithms are test-only. Pixel publication and successful statistics are atomic;
CPU-visible headless pixels require explicit readback.

## Discover The Current Structure

| Fact | Authoritative source |
| --- | --- |
| Package identity, dependencies, features and targets | [Cargo.toml](Cargo.toml) |
| Public front door and exported contracts | [src/lib.rs](src/lib.rs) and the reexported definitions |
| Headless glyph admission and placed Text adapter | [Prepared glyphs](src/text/prepared.rs) and [Text adapter](src/text_layout.rs) |
| Authored commands and normalization | [src/scene.rs](src/scene.rs), [src/command.rs](src/command.rs), and [src/style/](src/style/) |
| Planning, execution, and publication | [src/frame/](src/frame/), [src/renderer/](src/renderer/), [src/pass/](src/pass/), [src/gpu_transaction/](src/gpu_transaction/), and [src/backend/](src/backend/) |
| GPU resources, shaders, and internal raster engine | [src/resource/](src/resource/), [src/shader/](src/shader/), [src/shaders/](src/shaders/), and [src/vello_engine/](src/vello_engine/) |
| Surface lifecycle and explicit readback | [src/surface.rs](src/surface.rs) and [src/readback/](src/readback/) |
| Semantic and runtime capabilities, errors, and statistics | [src/capability.rs](src/capability.rs), [src/error.rs](src/error.rs), and [src/stats.rs](src/stats.rs) |
| Focused verification | [src/tests/](src/tests/), tracked local `#[cfg(test)]` modules, [src/reference/](src/reference/), and [tests/fixtures/](tests/fixtures/) |
| Live presented smoke target | [examples/render_window_smoke.rs](examples/render_window_smoke.rs) and its [Cargo.toml](Cargo.toml) target |

[README.md](README.md) links usage, contract and attribution documentation.

## Command Inventory

Select affected checks under the root process/resource policy; commands below
run from this crate directory.

Native host feature combinations are default, `render-window`, `render-web`, and
`render-window,render-web`. The independent optional `render-text` feature owns
immutable placed Text-layout consumption; its focused tests are in
`src/tests/text_layout_adapter.rs`. Native tests include real GPU execution and require
a compatible adapter; serial test threads do not bound GPU allocation. Select
`tests::platform::real_gpu_smoke_emits_no_uncaptured_error` for the initial 2×2
smoke rather than assuming the full GPU suite fits the host. Wasm coverage is
compile-only; browser execution/canvas presentation needs root integration
evidence. The native smoke example requires a graphical session and exits after
one direct and one GPU-graph frame.

```sh
CARGO_NET_OFFLINE=true cargo check -j 1 -p surgeist-render
CARGO_NET_OFFLINE=true cargo test -j 1 -p surgeist-render
CARGO_NET_OFFLINE=true cargo clippy -j 1 -p surgeist-render --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-render -- --check
```

```sh
CARGO_NET_OFFLINE=true cargo check -j 1 -p surgeist-render --target wasm32-unknown-unknown --features render-web --lib --tests
```

```sh
CARGO_NET_OFFLINE=true cargo run -j 1 -p surgeist-render --example render_window_smoke --features render-window
CARGO_NET_OFFLINE=true cargo run -j 1 -p surgeist-render --example render_window_smoke --features render-window,render-web
```
