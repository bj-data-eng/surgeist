# Getting started

Run one existing headless test to prove that this checkout can render a scene on
the GPU and explicitly copy the completed frame into CPU-visible pixels.

## Prerequisites

- A local checkout with a Rust toolchain capable of building the Rust 2024
  package and its pinned dependencies. See the [compatibility reference](reference.md#compatibility)
  for the root integration boundary.
- The dependencies already available to Cargo. The commands below use offline
  mode and do not download missing packages. The manifest also names the local
  `../surgeist-window` workspace dependency; keep that crate directory at its
  declared path for Cargo dependency resolution.
- A compatible WGPU adapter and device. Native headless execution does not
  require a window, but it still requires GPU support.

The [manifest](../Cargo.toml) owns the dependency and feature configuration.

## Render and read back a frame

1. Open a terminal in the `surgeist-render` crate directory, beside `Cargo.toml`.
2. Run the exact focused test:

   ```sh
   CARGO_NET_OFFLINE=true cargo test -p surgeist-render --lib tests::surface::headless_render_can_be_read_back -- --exact
   ```

3. Confirm that Cargo reports `tests::surface::headless_render_can_be_read_back`
   as `ok`, with `1 passed; 0 failed`. Other tests are filtered out; zero executed
   tests do not demonstrate success.

The [tracked test](../src/tests/surface.rs) creates a `Renderer` and a 4 × 4
headless `Surface`, fills a `Scene`, awaits rendering, then explicitly awaits
`Renderer::read_headless`. It checks that the surface is ready and the returned
`ImageBuffer` has 4 × 4 physical pixels, 64 RGBA bytes, and nonzero pixel data.
It does not create an image file or open a window.

An offline dependency-resolution failure means the prerequisites are incomplete.
A runtime capability failure means the host could not execute this GPU proof;
neither result is a passing skip. Consult the [capability and error
reference](reference.md#capabilities-and-errors) to distinguish unsupported
operations from unavailable device or surface capabilities.

Continue with the [how-to guide](how-to.md) to exercise live native presentation
or check the browser compilation target.
