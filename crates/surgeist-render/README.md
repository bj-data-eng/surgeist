# surgeist-render

A Rust library for Surgeist integrators who need GPU rendering of scenes onto
headless or presented surfaces. It owns scene validation, GPU execution, surface
resources, explicit headless readback, and render diagnostics.

The current implementation is GPU-only, with direct Vello and WGPU graph routes.
It requires a compatible GPU; browser execution and application/window lifecycle
belong to the integrating host. The current style-facing exports and intended
crate boundary are described in the [architecture guide](docs/explanation.md).

`FontData::prepare_outline` prepares an owned outline run without a GPU. It
supports simple glyf contours, static XY-translation composites and dense simple
gvar tuples at the actual face's exact F2Dot14 instance. Unscaled HarfBuzz contour interpretation is followed by checked
f64 size/UPEM scaling, skew, y reflection and local glyph placement, rounded once
into the retained canonical f32 path. Composite variation, nonidentity component
transforms, point attachment, metric overrides, sparse IUP, CFF/CFF2 and VARC
return typed unsupported capabilities; they never use static/default or
alternate-format outlines. Malformed present selection/variation metadata and
unrepresentable active delta arithmetic refuse the whole preparation. Complete
raw contour/flag/program extents are checked before decoding. Structurally valid
flag encodings outside the installed decoder's faithful domain return a typed
provider refusal. [Outline admission](docs/reference.md#outline-admission) defines
the bounded composite domain.

The result retains its font, executed `TextInstance`, original `TextGlyph` facts,
combined run-local path and derived `Ink` or source-proved `Empty` bounds. Explicit
`OutlineLimits` bound glyphs, path elements, temporary storage and retained logical
custody, including the font's boxed byte extent, conservative owner headers and
path reallocation overlap. They are not an allocator measurement or CPU-time
bound. `Scene::prepared_outline` shares the same immutable path with direct and
graph rendering as one NonZero fill, preserving run-local Color/Gradient/Image
paint and overlap semantics. It does not shape text or use advance boxes as ink.
Concrete positive emboldening appends a scaled local-x contour copy to the same
one-fill path under the [selected outline policy](../../docs/text-layout-contract.md#concrete-outline-synthesis).
Selected color/bitmap representations explicitly refuse.
Color/bitmap selection metadata is checked; unsupported paint/image payloads are
not executed or claimed to be validated. The authored `TextRun` path retains its
existing provider behavior, with `Unspecified` bounds permitted only by its direct
route. Broader outline/representation/synthesis support remains separate work.

The optional `render-text` feature adds `prepare_text_layout`, consuming the
actual immutable placed `surgeist_text::Layout` without reshaping or choosing
lines. Its result borrows that exact snapshot and owns the complete Scene. Text
qualifies the real face and effective normalized coordinates, including
source-proved default axes; unresolved faux-bold or skew refuse explicitly.
One `TextLayoutLimits` budget covers all runs, font copies, mapping buffers,
prepared paths and decoration command storage. The adapter publishes no prefix
when a later item refuses. Failure diagnostic allocations are outside those
successful-work limits. Font copies are shared only within that invocation;
the caller's original Text storage remains an input loan.

Glyph placement already includes Text's offsets and baselines. Existing
rectangle decorations follow glyphs; a nonidentity placement uses one admitted
decoration transform layer. Both glyph ink and decoration geometry receive
numeric placement qualification. Generic Scene copies retain owned paint
resources but carry no authority to join another Text snapshot. Both standalone
Text layout and explicit finalized-line placement use this boundary. A completed
Layout batch lends its actual placed Text result; fragment rectangles alone are
insufficient. Live computed-style authentication remains upstream. The adapter
preserves the existing outline capability limits.

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
