# Reference

## Package and public API

[Cargo.toml](../Cargo.toml) defines package `surgeist-render`, version `0.1.0`,
Rust edition 2024, and library import name `surgeist_render`. The public front
door is [src/lib.rs](../src/lib.rs); its reexports and their rustdoc own the
current source API.

| Area | Public entry points | Source |
| --- | --- | --- |
| Renderer and options | `Renderer`, `Options`, `Parameters`, `EffectQualityPolicy` | [Renderer](../src/renderer/mod.rs), [options](../src/renderer/options.rs), [surface parameters](../src/surface.rs) |
| Draw data | `Scene`, geometry, shapes, paints, images, layers, and positioned text runs | [Scene](../src/scene.rs), [public reexports](../src/lib.rs) |
| Surface and pixels | `Surface`, `SurfaceOptions`, `Attachment`, `ImageBuffer` | [Surfaces](../src/surface.rs), [images](../src/image.rs) |
| Support and diagnostics | `Capabilities`, `RuntimeCapabilities`, `Error`, `ErrorCode` | [Capabilities](../src/capability.rs), [errors](../src/error.rs) |
| Successful frame observations | `Stats`, `RenderRoute`, `EffectPrecision` | [Statistics](../src/stats.rs) |
| Current style-facing models | Background, border, outline, image-placement, mask, clip, and filter values | [Style module](../src/style/mod.rs), [public reexports](../src/lib.rs) |

The `Renderer` methods `new`, `create_surface`, `create_headless`,
`resume_surface`, `render`, and `read_headless` are asynchronous GPU operations.
`Renderer::read_headless` returns an
`ImageBuffer` containing tightly packed straight-alpha RGBA8 physical pixels from
the current complete headless publication. Rendering does not invoke this
readback implicitly.

## Capabilities and errors

`Capabilities::CURRENT` reports semantic support for authored rendering
operations and compiled surface availability. Its surface report supports
headless construction and enables its web-canvas flag only with both
`render-web` and the `wasm32` architecture. Native window attachment is separately
compiled behind `render-window`.

`Renderer::runtime_capabilities` reports availability or immutable facts about
the selected device and surface: surface format, supported effect precisions,
and maximum two-dimensional effect texture size. Cargo features select compiled
host adapters; enabling a feature does not establish runtime device support.

Unsupported semantic operations and unavailable runtime capabilities have
distinct typed diagnostics. Missing adapters, required formats, device limits,
and surface lifecycle availability do not authorize partial frame publication.
The [error API](../src/error.rs) defines the operation and reason payloads.

## Host features

| Configuration | Leaf boundary |
| --- | --- |
| Default features (`[]`) | Native headless GPU execution without a window. |
| `render-window` | Enables `surgeist-window` and native window attachment; live presentation requires its graphical host lifecycle. |
| `render-web` | Enables `wgpu/webgpu`; web-canvas support requires `wasm32`. The documented wasm leaf check compiles `--lib --tests`. |
| `render-window,render-web` on native | Additive compiled features; uses the same native live-window smoke target. |

Native WebCanvas surface creation returns a typed unsupported-platform
diagnostic. Real browser event-loop execution and presentation belong to root
integration, even after a successful wasm compilation.

## Compatibility

This crate's [manifest](../Cargo.toml) declares edition 2024 and intentionally has
no `rust-version` field. The existing integration guidance records Rust 1.97 as
the product integration floor; the [root manifest](../../../Cargo.toml) owns
the authoritative facade `rust-version`. Confirm that declaration when doing
integration work; this crate guide does not define a separate crate MSRV.

## Verification sources and commands

The focused tests are grouped in [src/tests/mod.rs](../src/tests/mod.rs), covering
models, authored-style normalization, frame planning, GPU passes, Vello raster
execution, surfaces/readback, and platform contracts. The [Ahem
fixture](../tests/fixtures/fonts/ahem/PROVENANCE.md) provides deterministic glyph
inputs. CPU pixel reference algorithms are compiled only for tests.

The repository command inventory is owned by [AGENTS.md](../AGENTS.md). With
dependencies already present, its standard Cargo commands can run offline:

```sh
CARGO_NET_OFFLINE=true cargo check -p surgeist-render
CARGO_NET_OFFLINE=true cargo test -p surgeist-render
CARGO_NET_OFFLINE=true cargo clippy -p surgeist-render --all-targets -- -F unsafe-code -D warnings
CARGO_NET_OFFLINE=true cargo fmt -p surgeist-render --check
```

Tests include real GPU work. No passing result is implied when the required
adapter is unavailable. The [how-to guide](how-to.md) owns the live native
presentation and wasm compilation procedures; the [getting-started
guide](getting-started.md) selects one focused headless proof.
