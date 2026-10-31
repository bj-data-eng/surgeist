# surgeist-render

A Rust library for Surgeist integrators who need GPU rendering of scenes onto
headless or presented surfaces. It owns scene validation, GPU execution, surface
resources, explicit headless readback, and render diagnostics.

The current implementation is GPU-only, with direct Vello and WGPU graph routes.
It requires a compatible GPU; browser execution and application/window lifecycle
belong to the integrating host. The current style-facing exports and intended
crate boundary are described in the [architecture guide](docs/explanation.md).

`FontData::prepare_glyph_run` admits immutable glyph resources without a GPU.
It retains qualified outline paths, COLR paint operations or decoded sbix PNG
pixels, with representation-derived `Ink` or proven `Empty` bounds. Direct and
graph drawing consume those same resources. The [glyph admission
reference](docs/reference.md#glyph-admission) defines the supported formats,
effective instances, numeric rules and explicit resource limits.

The optional `render-text` feature adds `prepare_text_layout`. It consumes the
exact immutable placed `surgeist_text::Layout`, preserving its font instances,
glyphs, lines and positions, including unrotated native upright and clockwise
or counterclockwise sideways runs. One `TextLayoutLimits` budget covers the complete
Scene; a later refusal publishes no prefix. Text shaping, line selection and
live computed-style authentication belong to their producing subsystems.

## Start

From this checkout, run the focused render-and-readback test:

```sh
CARGO_NET_OFFLINE=true cargo test -p surgeist-render --lib tests::surface::headless_render_can_be_read_back -- --exact
```

Expect one passing test: it renders a 4 × 4 scene on the GPU and reads back
64 RGBA bytes. See [getting started](docs/getting-started.md) for prerequisites
and the full first-success walkthrough.

## Documentation

| Guide | Purpose |
| --- | --- |
| [Getting started](docs/getting-started.md) | Render and read back your first headless frame. |
| [How-to](docs/how-to.md) | Exercise native presentation and check the browser compilation boundary. |
| [Reference](docs/reference.md) | Find public interfaces, features, diagnostics, and verification commands. |
| [Explanation](docs/explanation.md) | Understand execution routes, publication, precision, and ownership. |

## License and attribution

The project is licensed under the [MIT License](LICENSE). [Third-party
attribution](NOTICE.md) covers direct dependencies and bundled material.
