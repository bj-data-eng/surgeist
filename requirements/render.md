# surgeist::render Requirements

`surgeist::render` is the Vello-backed rendering boundary for Surgeist. It owns renderer setup, surface attachment, surface lifecycle, scene construction, scene submission, frame presentation, headless rendering, and renderer diagnostics.

The public API should be designed for use as `surgeist::render::*`. The module path supplies the layer name, so public type names should stay short and local: `Renderer`, `Surface`, `Scene`, `Paint`, `Image`, `Layer`, `Shape`, `Stroke`, `Shadow`, `Attachment`, `Options`, `SurfaceOptions`, `Parameters`, `Stats`, `Error`, and `Result`.

The fluent DSL requirements in [`render_dsl.md`](render_dsl.md) refine this
front-door API while preserving the renderer contracts described here.

## Scope

This module owns:

- Vello and `wgpu` integration.
- Renderer device and queue selection.
- Surface creation from native or web attachments.
- Surface resize, scale changes, suspend, resume, and destruction.
- Scene construction and scene submission.
- Conversion from Surgeist render primitives into Vello scene operations.
- Render parameters such as base color, surface size, antialiasing, and debug/profiling options.
- GPU resource lifetime, upload staging, cache invalidation, and diagnostics.
- Headless texture or image rendering for tests.

This module describes drawing and presentation. It does not own UI trees, CSS boxes, layout, paint order, z ordering, hit testing, input routing, text semantics, text editing, shape construction, accessibility semantics, animation timelines, or application commands.

Compound UI surfaces such as rounded boxes with arrows, borders, fills, and shadows belong in a surface or shape-construction layer above render. That layer should build paths and issue render commands. `surgeist::render` only draws the primitives it receives.

## Dependencies

Expected direct dependencies:

```text
surgeist-render
  -> vello
  -> glifo
  -> wgpu
  -> peniko
  -> kurbo
  -> raw-window-handle
  -> optional surgeist-window
```

`surgeist-render` may depend on `surgeist-window` for ergonomic native surface attachment. It must not require higher-level UI, document, widget, DSL, style, shape, or app crates.

`surgeist-render` must not depend on Parley or `surgeist-text` directly. Text rendering is accepted through a renderer-owned text-run contract. A separate bridge crate or an optional feature on `surgeist-text` may project Parley layouts into that contract.

The dependency direction is one-way:

```text
surgeist-text   -> parley/fontique
surgeist-render -> vello/wgpu
surgeist-text   --optional render projection--> surgeist-render
```

Text layout must be useful without a GPU; rendering must be useful without text.

## Naming

Public names are authored for the `surgeist::render` namespace:

- `Renderer` owns shared renderer state, including Vello renderer state and selected device/queue resources.
- `Surface` is a configured render destination, usually one native window surface, web canvas surface, or headless surface.
- `Scene` is the renderer-facing drawing list.
- `Paint` describes solid color, gradient, image, or future pattern paint.
- `Image` is an uploaded or uploadable image resource.
- `Layer` describes clip, transform, opacity, blend, and isolation state.
- `Shape` describes rectangles, rounded rectangles, paths, circles, and other drawable geometry.
- `Stroke` describes stroke width, joins, caps, dash pattern, and alignment.
- `Shadow` describes a renderer-level shadow operation over a supplied shape.
- `TextRun`, `TextGlyph`, `FontRef`, and `TextPaint` describe renderer-ready text input.
- `Attachment` describes a native or web object a surface can attach to.
- `Options` configures device and renderer behavior.
- `SurfaceOptions` configures one surface.
- `Parameters` configures one render operation.
- `Stats` reports renderer timing, resource, and scene facts.
- `Error` and `ErrorCode` are stable diagnostics.
- `Result<T>` is this module's result alias.

Avoid repeating `Render` in type names when the module path already supplies it. Prefer `surgeist::render::Scene` over `surgeist::render::RenderScene`.

## Coordinate Model

Rendering uses logical coordinates until surface submission. `Surface` owns conversion from logical size and scale factor to physical texture size.

The module may define or re-export minimal geometry primitives, but should prefer a shared Surgeist geometry crate once one exists:

```rust
pub struct Point { pub x: f64, pub y: f64 }
pub struct Size { pub width: f64, pub height: f64 }
pub struct Rect { pub origin: Point, pub size: Size }
pub struct Transform(/* private or shared affine representation */);
pub struct Radii { pub top_left: f64, pub top_right: f64, pub bottom_right: f64, pub bottom_left: f64 }
```

Vello and `kurbo` types may appear in advanced extension APIs, but front-door APIs should use Surgeist types so app code does not become coupled to Vello naming.

## Core API

```rust
pub struct Renderer {
    // private wgpu instance, adapter, device, queue, renderer cache
}

pub struct Surface {
    // private native, web, or headless surface state
}

pub struct Scene {
    // private command storage or Vello scene storage
}

impl Renderer {
    pub async fn new(options: Options) -> Result<Self>;
    pub fn create_surface(&mut self, attachment: Attachment, options: SurfaceOptions) -> Result<Surface>;
    pub fn create_headless(&mut self, size: Size, scale: f64) -> Result<Surface>;
    pub fn render(&mut self, surface: &mut Surface, scene: &Scene, parameters: Parameters) -> Result<Stats>;
    pub fn resume_surface(&mut self, surface: &mut Surface, attachment: Attachment) -> Result<()>;
}

impl Surface {
    pub fn resize(&mut self, size: Size, scale: f64) -> Result<()>;
    pub fn suspend(&mut self) -> Result<()>;
    pub fn resume(&mut self, attachment: Attachment) -> Result<()>;
    pub fn size(&self) -> Size;
    pub fn scale(&self) -> f64;
}

impl Scene {
    pub fn new() -> Self;
    pub fn clear(&mut self);
    pub fn fill(&mut self, shape: impl Into<Shape>, paint: impl Into<Paint>) -> &mut Self;
    pub fn stroke(&mut self, shape: impl Into<Shape>, stroke: Stroke, paint: impl Into<Paint>) -> &mut Self;
    pub fn shadow(&mut self, shape: impl Into<Shape>, shadow: Shadow) -> &mut Self;
    pub fn image(&mut self, image: Image, rect: Rect, fit: ImageFit) -> &mut Self;
    pub fn text_run(&mut self, run: TextRun<'_>) -> &mut Self;
    pub fn layer(&mut self, layer: Layer, children: impl FnOnce(&mut Scene)) -> &mut Self;
    pub fn transform(&mut self, transform: Transform, children: impl FnOnce(&mut Scene)) -> &mut Self;
    pub fn clip(&mut self, shape: impl Into<Shape>, children: impl FnOnce(&mut Scene)) -> &mut Self;
    pub fn stats(&self) -> Stats;
}
```

`Renderer` is shared render backend state. `Surface` is per-window, per-web-canvas, or per-headless-destination state. `Scene` is the renderer-facing drawing list and drawing API.

`Attachment` should accept a `surgeist::window::Handle` when the `window` feature is enabled and a web canvas or WebGPU-compatible surface source when the `web` feature is enabled.

## Strokes

`Stroke` is the renderer-level outline operation for paths and shapes. It wraps Vello/kurbo stroke capabilities and adds Surgeist stroke alignment as a public contract.

```rust
pub struct Stroke {
    pub width: f64,
    pub join: LineJoin,
    pub start_cap: LineCap,
    pub end_cap: LineCap,
    pub miter_limit: f64,
    pub dash: Option<Dash>,
    pub align: StrokeAlign,
}

pub enum LineJoin {
    Miter,
    Round,
    Bevel,
}

pub enum LineCap {
    Butt,
    Round,
    Square,
}

pub enum StrokeAlign {
    Center,
    Inside,
    Outside,
}
```

Rules:

- Center-aligned strokes lower directly to Vello/kurbo stroke operations.
- Inside and outside stroke alignment are first-class Surgeist render semantics, even when they require renderer-side shape/path lowering before Vello submission.
- Upstream layout, text, flexbox, box model, and style code should not hand-roll outline geometry to compensate for stroke alignment.
- Inside and outside alignment must preserve the source shape's visual intent for rectangles, rounded rectangles, circles, ellipses, and paths.
- Unsupported alignment cases must produce explicit diagnostics or deterministic degraded output, not silent geometry changes.
- Dash, cap, join, and miter behavior should map to Vello/kurbo where possible.
- Stroke alignment is about drawing. It does not change layout bounds unless an upstream layout/style layer chooses to account for the visual overflow.

Current limitation:

- Inside and outside alignment for arbitrary paths must fail with `UnsupportedBackend` until path offsetting is owned by a dedicated shape/lowering layer. Built-in rectangles, rounded rectangles, circles, and ellipses remain in scope for the first implementation.

## Scene Encoding

```rust
impl Scene {
    pub fn fill(&mut self, shape: impl Into<Shape>, paint: impl Into<Paint>) -> &mut Self;
    pub fn stroke(&mut self, shape: impl Into<Shape>, stroke: Stroke, paint: impl Into<Paint>) -> &mut Self;
    pub fn shadow(&mut self, shape: impl Into<Shape>, shadow: Shadow) -> &mut Self;
    pub fn image(&mut self, image: Image, rect: Rect, fit: ImageFit) -> &mut Self;
    pub fn text_run(&mut self, run: TextRun<'_>) -> &mut Self;
    pub fn layer(&mut self, layer: Layer, children: impl FnOnce(&mut Scene)) -> &mut Self;
    pub fn transform(&mut self, transform: Transform, children: impl FnOnce(&mut Scene)) -> &mut Self;
    pub fn clip(&mut self, shape: impl Into<Shape>, children: impl FnOnce(&mut Scene)) -> &mut Self;
}
```

Rules:

- Scene encoding should be deterministic.
- Scene storage must be reusable across frames.
- Scene drawing methods should be chainable where that improves readability.
- Scene facts should be inspectable without submitting to a renderer.
- Renderer-facing commands are visual facts, not app meaning.
- The renderer should accept already-resolved paint, shape, transform, clip, image, and text layout facts.
- CSS/style resolution belongs above this crate.
- Layout belongs above this crate.
- Complex surface outlines belong above this crate.
- Child ordering belongs above this crate.

## Layers And Clips

Vello exposes both clip paths and isolated clip/layer operations. `surgeist::render` should keep that distinction internally while presenting a small public API.

Rules:

- Plain geometric clipping should lower to the cheapest correct Vello clip path operation.
- Clips combined with opacity, blend, mask, filter, shadow, or glyph color blending should lower to an isolated layer when required for correctness.
- `Layer` should support clip, transform, opacity, blend mode, optional mask, and optional filter.
- The public API should prevent unbalanced push/pop layer states by using scoped scene methods such as `layer`, `clip`, and `transform`.
- The renderer must preserve deterministic paint order across nested layers and clips.
- Diagnostics should report unsupported layer, mask, filter, or blend combinations rather than silently flattening them.

Current limitation:

- General `Layer::mask` lowering must fail with `UnsupportedBackend` until Surgeist has a stable mask/effect contract. Plain geometric clipping remains available through `clip`.
- General `Layer::filter` lowering must fail with `UnsupportedBackend` until Surgeist has a stable effect-layer contract. Shape-specific shadow and blur primitives remain separate render operations.

## Images

Vello and peniko provide image source, sampling, quality, and extend behavior. `surgeist::render::Image` should wrap those capabilities without leaking backend image-cache details.

Rules:

- Image drawing should preserve source size, target rectangle, fit mode, sampling quality, and extend mode.
- `ImageFit` is a Surgeist convenience that lowers to transform, clip, and sampling operations.
- Image uploads and image cache lifetime belong to `Renderer`.
- Repeated use of the same image data or image handle should not cause repeated uploads in warm frames.
- Image statistics should report uploads and cache reuse.

## Shadows

`Shadow` is a renderer primitive over a supplied shape, not a CSS box model.

```rust
pub struct Shadow {
    pub offset: Point,
    pub blur: f64,
    pub spread: f64,
    pub paint: Paint,
}
```

Rules:

- Shadows are outer shadows in the first API version.
- `blur` is a CSS-like blur radius at the Surgeist API. Vello lowering should convert to renderer standard deviation internally, initially using `std_dev = blur * 0.5`.
- `spread` expands the supplied shape before blur when the shape supports expansion.
- Rounded rectangle shadows should lower to Vello blurred rounded rectangles when possible.
- Arbitrary shape or layer shadows may lower to Vello drop-shadow filters when available.
- Unsupported shadow cases must produce diagnostics or explicit degraded output, not silent contract changes.
- Inset shadows belong to a future shape/surface design, not the first render primitive.

Current limitation:

- Rounded rectangle shadows require uniform corner radii until Surgeist has shape-level lowering for non-uniform blurred rounded rectangles.

## Text Bridge

Text rendering is an integration point, not a reason for render and text to merge.

`surgeist::render` defines the renderer-facing text-run input needed by Vello/glifo. It does not define how text is shaped, wrapped, edited, selected, or hit-tested.

```rust
pub struct TextRun<'a> {
    pub font: FontRef<'a>,
    pub size: f32,
    pub transform: Transform,
    pub paint: TextPaint,
    pub glyphs: &'a [TextGlyph],
}

pub struct TextGlyph {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub advance: f32,
}

pub struct FontRef<'a> {
    // private or backend-compatible font reference
}

pub struct TextPaint {
    pub fill: Paint,
}
```

Rules:

- `surgeist-text` owns shaping, bidi, line breaking, selection geometry, cursor geometry, and editable text intent.
- `surgeist-render` owns drawing glyph runs once text has been prepared.
- The render crate must not require Parley or `surgeist-text`.
- `Scene::text_run` accepts `TextRun` values and lowers them into Vello/glifo during render submission.
- `surgeist-text` or a bridge crate may provide convenience projection from Parley-backed layouts into `TextRun` values.
- The bridge should iterate prepared glyph runs and decorations from text layout output and encode them into Vello without re-shaping.
- Rendering must not mutate text layout state.
- Selection, cursor, and decoration drawing are ordinary render commands supplied from text/editor state.
- Glyph atlas and glyph resource caching belong to Vello/glifo-backed render internals.
- Render statistics should expose glyph/cache/upload facts without exposing glifo cache keys as the public Surgeist API.

## Surfaces

`Surface` owns render destination state and presentation policy.

Rules:

- Native surfaces are created from live window handles, not detached raw handle values.
- Web canvas surfaces are first-class where Vello/wgpu support allows them.
- Surface lost, out-of-memory, timeout, outdated, and unsupported cases must map to stable `ErrorCode` values.
- Vello or wgpu out-of-memory failures during scene submission must map to `ErrorCode::SurfaceOutOfMemory`.
- Resize and scale changes reconfigure surfaces before the next frame.
- Suspended surfaces release or pause surface resources as required by the platform.
- Presented native and web surfaces resume through `Renderer::resume_surface` because reattachment needs renderer backend state.
- A destroyed native window invalidates the surface attached to it.
- A `Surface` is associated with one live attachment at a time.

## Headless Rendering

Headless rendering is required for tests and tooling.

Rules:

- A headless surface can render a scene to an image buffer or texture without `winit`.
- Headless tests should be able to compare scene facts and optional pixel output.
- Pixel tests must allow platform/backend tolerances.
- Headless rendering should use the same scene encoding path as surface rendering.

## Statistics

`Stats` should make renderer behavior inspectable without leaking backend internals as public APIs.

```rust
pub struct Stats {
    pub frame_time: Duration,
    pub encode_time: Duration,
    pub render_time: Duration,
    pub present_time: Duration,
    pub commands: usize,
    pub fills: usize,
    pub strokes: usize,
    pub shadows: usize,
    pub images: usize,
    pub glyphs: usize,
    pub layers: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub uploaded_bytes: u64,
}
```

Rules:

- `Stats` is diagnostic and test-facing.
- Exact backend counters may expand over time, but core frame, command, shadow, image, glyph, cache, upload, and timing facts should remain available.
- Stats should be useful for detecting accidental fallback paths, repeated uploads, runaway scene growth, and text/render cache churn.

## Errors

```rust
pub type Result<T> = std::result::Result<T, Error>;

pub struct Error {
    pub code: ErrorCode,
    pub message: String,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

pub enum ErrorCode {
    AdapterUnavailable,
    DeviceCreateFailed,
    RendererCreateFailed,
    SurfaceCreateFailed,
    SurfaceConfigureFailed,
    SurfaceLost,
    SurfaceOutOfMemory,
    SurfaceTimeout,
    SurfaceOutdated,
    SurfaceUnavailable,
    InvalidInput,
    ImageUploadFailed,
    RenderFailed,
    PresentFailed,
    UnsupportedBackend,
}
```

Error codes must remain stable. Display messages may improve over time.

## Tests

Required contract tests:

- `Renderer` can be constructed in fake or headless mode.
- `Surface` preserves logical size, physical size, and scale factor.
- Resize and scale changes reconfigure surfaces in the expected order.
- Surface loss maps to stable diagnostics.
- Scene encoding preserves fill, stroke, shadow, image, clip, transform, layer, opacity, and blend commands.
- Plain clips lower without unnecessary isolated layers where possible.
- Opacity, blend, mask, filter, and shadow layers lower to isolated layers when required for correctness.
- Image drawing preserves fit, sampling quality, extend mode, and warm-frame upload reuse.
- Shadow lowering preserves offset, blur, spread, paint, and shape identity.
- Scene encoding is deterministic for equivalent command sequences.
- Headless rendering uses the same scene path as surface rendering.
- Text bridge encodes prepared glyph runs without invoking text layout.
- `Stats` reports command, shadow, glyph, cache, upload, and timing facts.
- Render errors include stable `ErrorCode` values.

Required smoke tests:

- Create one native window through `surgeist::window`.
- Create a render surface from its `Handle`.
- Render a simple scene after `redraw_requested`.
- Resize the native window and re-render at the new surface size.
- Render one headless scene and verify non-empty output.

## First Milestone

Create a minimal Vello-backed example that:

1. Opens a native window through `surgeist::window`.
2. Creates a `surgeist::render::Renderer`.
3. Creates a `Surface` from the window `Handle`.
4. Encodes a scene with one shadow, one filled shape, one stroked shape, and one clipped layer.
5. Renders on redraw.
6. Reconfigures on resize and scale change.
7. Includes headless scene encoding tests.
