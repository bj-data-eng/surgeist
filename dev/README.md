# surgeist-dev

Native integration harness for the early Surgeist foundation modules.

This crate is intentionally practical: it opens real native windows through
`surgeist::window`, creates real Vello-backed surfaces through
`surgeist::render`, lays out real Parley-backed text through `surgeist::text`,
and renders the result as ordinary text runs and drawing commands.

Run it with:

```sh
just surgeist-harness
```

Memory isolation modes:

```sh
just surgeist-harness window
just surgeist-harness renderer
just surgeist-harness surface
just surgeist-harness empty-render
just surgeist-harness render
just surgeist-harness cpu-empty-render
just surgeist-harness cpu-render
just surgeist-harness cpu-full
just surgeist-harness text-system
just surgeist-harness text-layout
just surgeist-harness full
```

The `cpu-*` modes are diagnostic probes for Vello's CPU pipeline stages. They
are not expected to reduce memory versus the default renderer path.

Keyboard controls:

- `1` through `5`: switch directly to a scenario.
- Arrow keys: cycle scenarios.
- `Escape`: exit.

Scenarios:

- Text basics: wrapping, spans, color, underline, strikethrough, line and cluster overlays.
- Bidi, cursor, selection: mixed-direction text, selection geometry, and cursor geometry.
- Inline boxes: in-flow and out-of-flow inline box projection.
- Render primitives: gradients, shadows, stroke alignment, paths, caps, images, clipping, opacity, and blend.
- Window and input state: live metrics, scale, focus, and recent native event log.

The Parley crate documentation points to its upstream examples for rendering
usage. This harness mirrors the same correctness categories at the Surgeist API
boundary: shaped glyph runs, line breaking, bidi, cursor geometry, selection
geometry, decorations, and inline boxes.
