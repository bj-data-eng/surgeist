# surgeist::render DSL Requirements

`surgeist::render` exposes a two-tier authoring surface over Vello-backed
rendering. The DSL should make render setup and scene construction concise,
typed, inspectable, and pleasant without changing the renderer contract. The
render layer still owns devices, surfaces, scene lowering, presentation,
headless output, and diagnostics. Higher layers own layout, UI trees, style
resolution, interaction, shape semantics, animation, and app commands.

The public API is designed for the `surgeist::render` namespace. Names stay
short because the module path supplies the layer: `renderer`, `scene`,
`surface`, `window`, `headless`, `frame`, `rect`, `round`, `radii`, `circle`,
`ellipse`, `path`, `stroke`, `dash`, `layer`, `rgba`, `linear`,
`radial`, `sweep`, `image`, `font`, `text`, `Renderer`, `Surface`, `Scene`, `Paint`,
`Shape`, `Stroke`, `Layer`, `Frame`, `Options`, and `Parameters`.

## Design Target

Renderer and surface setup are fluent but still explicit:

```rust
use surgeist::{render, window};

async fn make_renderer() -> render::Result<render::Renderer> {
    render::renderer()
        .aa(render::Aa::Area)
        .build()
        .await
}

fn attach(
    renderer: &mut render::Renderer,
    win: &mut window::Ready<'_>,
) -> render::Result<render::Surface> {
    renderer.surface(
        render::window(win.handle()?)
            .metrics(win.metrics())
            .present(render::PresentMode::Auto),
    )
}
```

Scene authoring is compact and scoped:

```rust
let panel_shadow = render::shadow(render::point(0, 12), 24)
    .color(render::rgba(0.0, 0.0, 0.0, 0.38))
    .build()?;

let scene = render::scene(|s| {
    s.shadow(
        render::round(40, 40, 220, 120).r(18),
        panel_shadow.clone(),
    )
    .fill(
        render::round(40, 40, 220, 120).r(18),
        render::rgba(0.96, 0.97, 1.0, 1.0),
    )
    .stroke(
        render::round(40, 40, 220, 120).r(18),
        render::stroke(2).inside(),
        render::rgb(0.15, 0.33, 0.64),
    )
    .clip(render::rect(300, 48, 180, 96), |s| {
        s.layer(render::layer().opacity(0.75), |s| {
            s.fill(render::rect(280, 28, 220, 140), render::rgb(1.0, 0.68, 0.26));
        });
    });
});
```

Submission keeps render lifecycle visible:

```rust
renderer
    .frame(&mut surface)
    .clear(render::transparent())
    .debug(show_render_debug)
    .draw(&scene)?;
```

The DSL is a front door over existing render types. A caller can still use
`Renderer::new`, `Renderer::create_surface`, `Scene::new`, `Surface::resize`,
and other core methods directly.

## Principles

- Builders are inert values until passed to `Renderer`, `Scene`, or a scoped
  callback.
- Builder output is inspectable. Shape builders lower to `Shape`, paint
  builders lower to `Paint`, stroke builders lower to `Stroke`, layer builders
  lower to `Layer`, surface builders lower to `Attachment` plus
  `SurfaceOptions`, and frame builders lower to `Parameters`.
- Fluent methods are short and chainable. Variants are expressed through typed
  values instead of verbose method families.
- Logical coordinates are the default. Physical units are explicit in type
  names.
- Scene authoring is deterministic. The same builder sequence produces the same
  command sequence.
- Scoped scene helpers prevent unbalanced layer and clip state.
- Renderer and surface setup remain explicit enough that resource ownership,
  suspension, resume, resize, and presentation are visible in app code.
- Render DSL helpers describe renderer-facing visual facts.

## Front Door

The DSL may live in `dsl.rs` internally and is exported from `lib.rs`:

```rust
pub use dsl::{
    circle, dash, ellipse, font, font_data, headless, image, layer,
    linear, path, point, radial, radii, rect, renderer, rgba, rgb, round, scene,
    shadow, size, stroke, sweep, text, transparent, window, Aa, Frame, Gradient,
    DashBuilder, GradientBuilder, Headless, ImageBuilder, LayerBuilder,
    PathBuilder, RendererBuilder, ShadowBuilder, ShapeBuilder, StrokeBuilder,
    SurfaceTarget, TextBuilder, WindowSurface,
};
```

`lib.rs` also re-exports stable contracts from render modules, including
`Renderer`, `Surface`, `Scene`, `Paint`, `Image`, `Layer`, `Shape`, `Stroke`,
`Shadow`, `Attachment`, `Options`, `SurfaceOptions`, `Parameters`, `Stats`,
`ImageBuffer`, `FontRef`, `FontData`, `TextGlyph`, `TextPaint`, `TextRun`,
`Error`, and `Result`.

`Aa` is a short public alias or enum for `Antialiasing`. The underlying
`Antialiasing` type may remain exported for explicit code; the DSL should prefer
`Aa` in examples and builder methods.

## Renderer Builder

`renderer()` creates a `RendererBuilder`.

```rust
pub fn renderer() -> RendererBuilder;

pub struct RendererBuilder {
    // private Options
}
```

Required methods:

```rust
impl RendererBuilder {
    pub fn new() -> Self;
    pub fn aa(mut self, aa: Aa) -> Self;
    pub fn cpu(mut self, enabled: bool) -> Self;
    pub fn debug(mut self, enabled: bool) -> Self;
    pub fn options(&self) -> Options;
    pub fn build(self) -> impl Future<Output = Result<Renderer>>;
}
```

Rules:

- `aa` configures antialiasing through the existing render options.
- `cpu` exposes the existing diagnostic CPU pipeline option. It is not described
  as a lower-memory mode.
- `debug` enables renderer-level debug behavior.
- `options()` returns the exact `Options` value that `build()` will use.
- `build()` is a convenience over `Renderer::new(options)`.

## Surface Builders

Surface builders collect an `Attachment` and `SurfaceOptions`, then lower into a
single inspectable `SurfaceTarget`.

```rust
pub fn window(handle: window::Handle) -> WindowSurface;
pub fn headless(size: impl Into<Size>) -> Headless;

pub struct WindowSurface {
    // private Attachment + SurfaceOptions
}

pub struct Headless {
    // private SurfaceOptions
}

pub struct SurfaceTarget {
    // private Attachment + SurfaceOptions
}
```

Required methods:

```rust
impl WindowSurface {
    pub fn metrics(mut self, metrics: &window::Metrics) -> Self;
    pub fn size(mut self, size: impl Into<Size>) -> Self;
    pub fn scale(mut self, scale: f64) -> Self;
    pub fn present(mut self, mode: PresentMode) -> Self;
    pub fn format(mut self, format: Format) -> Self;
    pub fn attachment(&self) -> Attachment;
    pub fn options(&self) -> SurfaceOptions;
}

impl Headless {
    pub fn scale(mut self, scale: f64) -> Self;
    pub fn options(&self) -> SurfaceOptions;
}

impl Renderer {
    pub fn surface(&mut self, surface: impl Into<SurfaceTarget>) -> Result<Surface>;
}

impl SurfaceTarget {
    pub fn attachment(&self) -> Attachment;
    pub fn options(&self) -> SurfaceOptions;
}

impl From<WindowSurface> for SurfaceTarget;
impl From<Headless> for SurfaceTarget;
```

Rules:

- `WindowSurface::metrics` copies logical size and scale from
  `surgeist::window::Metrics`.
- Native window attachment is available behind the existing `window` feature.
- `window(handle)` and `WindowSurface` are available behind the existing
  `window` feature.
- Web canvas attachment may use a parallel `canvas(...)` builder when the web
  target is implemented.
- `Renderer::surface` lowers to `Renderer::create_surface`.
- Headless surfaces lower to `Attachment::Headless` and `SurfaceOptions`.
- Headless surfaces use `Format::Rgba8` in the first milestone because the
  current backend renders headless output through Rgba8 storage textures.
- `SurfaceTarget` is the inspectable lowering value for tests and advanced code.

## Surface Lifecycle Helpers

The DSL keeps resource lifecycle visible while adding short helpers for common
window integration.

Required methods:

```rust
impl Surface {
    pub fn resize_metrics(&mut self, metrics: &window::Metrics) -> Result<()>;
}

impl Renderer {
    pub fn resume(&mut self, surface: &mut Surface, target: impl Into<SurfaceTarget>) -> Result<()>;
    pub fn resizing(&mut self, surface: &mut Surface, active: bool) -> Result<()>;
    pub fn read(&mut self, surface: &Surface) -> Result<ImageBuffer>;
}
```

Rules:

- `Surface::resize_metrics` lowers to `Surface::resize(metrics.logical_size,
  metrics.scale_factor)`.
- `resize_metrics` is available behind the existing `window` feature.
- `Renderer::resume` lowers to `Renderer::resume_surface` and uses only the
  target attachment. Resize remains explicit.
- `Renderer::resizing` lowers to `Renderer::set_surface_resizing`.
- `Renderer::read` is a short alias for `Renderer::read_headless`.
- Readback succeeds only for rendered headless surfaces and preserves the
  existing `UnsupportedBackend` diagnostic for other surface kinds.
- Core lifecycle APIs remain available for code that prefers the explicit
  method names.

## Frame Builder

`Renderer::frame(&mut surface)` creates a scoped submission builder.

```rust
pub struct Frame<'a> {
    // private &mut Renderer + &mut Surface + Parameters
}
```

Required methods:

```rust
impl Renderer {
    pub fn frame<'a>(&'a mut self, surface: &'a mut Surface) -> Frame<'a>;
}

impl Frame<'_> {
    pub fn clear(mut self, color: impl Into<Color>) -> Self;
    pub fn debug(mut self, enabled: bool) -> Self;
    pub fn parameters(&self) -> Parameters;
    pub fn draw(self, scene: &Scene) -> Result<Stats>;
}
```

Rules:

- `clear` sets `Parameters::base_color`.
- `debug` sets `Parameters::debug`.
- `parameters()` returns the exact parameters that `draw()` will use.
- `draw()` lowers to `Renderer::render(surface, scene, parameters)`.
- The frame builder does not store persistent frame state.

## Scene Authoring

`scene(|s| { ... })` creates and returns a `Scene`.

```rust
pub fn scene(children: impl FnOnce(&mut Scene)) -> Scene;
```

`Scene` keeps the existing methods and may gain small aliases only when they
make fluent authoring more regular.

Required helpers:

```rust
impl Scene {
    pub fn push(&mut self, children: impl FnOnce(&mut Scene)) -> &mut Self;
    pub fn group(&mut self, children: impl FnOnce(&mut Scene)) -> &mut Self;
}
```

Rules:

- `scene` starts from `Scene::new`.
- `push` runs a closure against the same scene and returns `self`.
- `group` is an alias for an identity layer only if it remains useful after
  implementation review. Otherwise omit it.
- `fill`, `stroke`, `shadow`, `image`, `text_run`, `layer`, `transform`, and
  `clip` remain the core drawing vocabulary.
- Scene methods continue to return `&mut Self`.

## Geometry Helpers

Geometry helpers create existing render geometry types:

```rust
pub fn point(x: impl Into<f64>, y: impl Into<f64>) -> Point;
pub fn size(width: impl Into<f64>, height: impl Into<f64>) -> Size;
pub fn rect(x: impl Into<f64>, y: impl Into<f64>, width: impl Into<f64>, height: impl Into<f64>) -> Rect;
pub fn radii(all: impl Into<f64>) -> Radii;
```

Required builder helpers:

```rust
pub fn round(x: impl Into<f64>, y: impl Into<f64>, width: impl Into<f64>, height: impl Into<f64>) -> ShapeBuilder;
pub fn circle(center: impl Into<Point>, radius: impl Into<f64>) -> Shape;
pub fn ellipse(center: impl Into<Point>, radii: impl Into<Size>) -> Shape;
pub fn path() -> PathBuilder;
```

`ShapeBuilder` covers rounded rectangles:

```rust
impl ShapeBuilder {
    pub fn r(mut self, radius: impl Into<f64>) -> Self;
    pub fn radii(mut self, radii: Radii) -> Self;
    pub fn build(self) -> Shape;
}

impl From<ShapeBuilder> for Shape;
```

`PathBuilder` covers path construction:

```rust
impl PathBuilder {
    pub fn move_to(mut self, point: impl Into<Point>) -> Self;
    pub fn line_to(mut self, point: impl Into<Point>) -> Self;
    pub fn quad_to(mut self, control: impl Into<Point>, point: impl Into<Point>) -> Self;
    pub fn cubic_to(mut self, a: impl Into<Point>, b: impl Into<Point>, point: impl Into<Point>) -> Self;
    pub fn close(mut self) -> Self;
    pub fn build(self) -> Path;
}

impl From<PathBuilder> for Path;
impl From<PathBuilder> for Shape;
```

Rules:

- `rect` returns `Rect`, not `Shape`, so it remains useful for image targets and
  clipping.
- `round` returns a shape builder because corner radii are usually chained.
- Helpers validate through the existing render validation path during scene
  encoding or image/surface creation.

## Paint Helpers

Paint helpers lower to `Color`, `Paint`, `Gradient`, or `Image`.

```rust
pub fn transparent() -> Color;
pub fn rgb(r: impl Into<f32>, g: impl Into<f32>, b: impl Into<f32>) -> Color;
pub fn rgba(r: impl Into<f32>, g: impl Into<f32>, b: impl Into<f32>, a: impl Into<f32>) -> Color;
pub fn linear(start: impl Into<Point>, end: impl Into<Point>) -> GradientBuilder;
pub fn radial(center: impl Into<Point>, radius: impl Into<f64>) -> GradientBuilder;
pub fn sweep(center: impl Into<Point>) -> GradientBuilder;
pub fn image(size: impl Into<Size>, bytes: impl Into<Arc<[u8]>>) -> ImageBuilder;
```

`GradientBuilder`:

```rust
impl GradientBuilder {
    pub fn stop(mut self, offset: f32, color: impl Into<Color>) -> Self;
    pub fn build(self) -> Gradient;
}

impl From<GradientBuilder> for Gradient;
impl From<GradientBuilder> for Paint;
```

`ImageBuilder`:

```rust
impl ImageBuilder {
    pub fn quality(mut self, quality: ImageQuality) -> Self;
    pub fn extend(mut self, extend: Extend) -> Self;
    pub fn build(self) -> Result<Image>;
}
```

Rules:

- `rgb` sets alpha to `1.0`.
- `rgba` is the canonical explicit color helper.
- Gradient builders preserve stop order.
- Image byte length validation remains strict and uses existing image errors.
- `image(size, bytes)` returns an inert builder so image sampling options can be
  chained before validation.
- Scene drawing keeps the existing `Scene::image(image, rect, fit)` method.

## Stroke Helpers

`stroke(width)` creates a `StrokeBuilder`.

```rust
pub fn stroke(width: impl Into<f64>) -> StrokeBuilder;
pub fn dash(intervals: &'static [f64]) -> DashBuilder;

pub struct StrokeBuilder {
    // private Stroke
}
```

Required methods:

```rust
impl StrokeBuilder {
    pub fn join(mut self, join: LineJoin) -> Self;
    pub fn cap(mut self, cap: LineCap) -> Self;
    pub fn start(mut self, cap: LineCap) -> Self;
    pub fn end(mut self, cap: LineCap) -> Self;
    pub fn miter(mut self, limit: f64) -> Self;
    pub fn dash(mut self, dash: impl Into<Dash>) -> Self;
    pub fn align(mut self, align: StrokeAlign) -> Self;
    pub fn center(self) -> Self;
    pub fn inside(self) -> Self;
    pub fn outside(self) -> Self;
    pub fn build(self) -> Stroke;
}

impl From<StrokeBuilder> for Stroke;
```

`DashBuilder`:

```rust
impl DashBuilder {
    pub fn offset(mut self, offset: f64) -> Self;
    pub fn build(self) -> Dash;
}

impl From<DashBuilder> for Dash;
```

Rules:

- `cap` sets both start and end caps.
- `start` and `end` set caps independently.
- `center`, `inside`, and `outside` are short aliases over `align`.
- Dash support remains renderer-level. UI-perfect border dash distribution
  belongs in `surgeist-shape` or a style/shape layer above render.

## Layer Helpers

`layer()` creates a `LayerBuilder`.

```rust
pub fn layer() -> LayerBuilder;
```

Required methods:

```rust
impl LayerBuilder {
    pub fn clip(mut self, shape: impl Into<Shape>) -> Self;
    pub fn transform(mut self, transform: Transform) -> Self;
    pub fn opacity(mut self, opacity: f32) -> Self;
    pub fn blend(mut self, blend: BlendMode) -> Self;
    pub fn mask(mut self, shape: impl Into<Shape>) -> Self;
    pub fn blur(mut self, radius: f64) -> Self;
    pub fn build(self) -> Layer;
}

impl From<LayerBuilder> for Layer;
```

Rules:

- `blur` lowers to `Filter::Blur`.
- `mask` and `blur` are contract-setting helpers. In the first milestone they
  preserve the existing renderer behavior: unsupported mask/filter lowering
  produces `UnsupportedBackend` during render submission.
- `Scene::clip` remains the preferred helper for plain geometric clipping.
- `Scene::layer` remains the preferred helper for opacity, blend, mask, filter,
  or combined isolation.

## Shadow Helpers

`shadow(offset, blur)` creates a `ShadowBuilder`.

```rust
pub fn shadow(offset: impl Into<Point>, blur: impl Into<f64>) -> ShadowBuilder;
```

Required methods:

```rust
impl ShadowBuilder {
    pub fn spread(mut self, spread: f64) -> Self;
    pub fn paint(mut self, paint: impl Into<Paint>) -> Self;
    pub fn color(mut self, color: impl Into<Color>) -> Self;
    pub fn build(self) -> Result<Shadow>;
}
```

Rules:

- Shadow builders require explicit paint through `paint` or `color`.
- `build()` returns `InvalidInput` if paint is missing, unless implementation
  chooses a type-state builder that makes missing paint impossible.
- Scene helpers may accept `ShadowBuilder` only if they can surface missing
  paint as a typed render error before submission. Otherwise callers explicitly
  call `build()`.
- `color` is a convenience over `paint(Paint::Color(color))`.
- Shadow support remains the renderer primitive described in `render.md`.

## Text Helpers

Text helper names remain renderer-facing. They do not shape text.

```rust
pub fn font(id: u64) -> FontRef<'static>;
pub fn font_data(bytes: Vec<u8>, index: u32) -> FontData;
pub fn text(font: FontRef<'_>, size: f32, glyphs: &[TextGlyph]) -> TextBuilder<'_>;
```

Required methods:

```rust
impl TextBuilder<'_> {
    pub fn transform(mut self, transform: Transform) -> Self;
    pub fn fill(mut self, paint: impl Into<Paint>) -> Self;
    pub fn build(self) -> TextRun<'_>;
}

impl From<TextBuilder<'_>> for TextRun<'_>;
```

Rules:

- `font(id)` lowers to `FontRef::new(id)`.
- `font_data(bytes, index)` lowers to `FontData::from_bytes(bytes, index)`.
- `surgeist-render` accepts prepared glyphs only.
- Text shaping, wrapping, bidi, selection, cursor geometry, and editable text
  intent remain in `surgeist-text` or higher layers.
- Text decorations are ordinary scene commands supplied by the text/editor
  layer.

## Testing Contract

Required tests:

- DSL helpers lower to the same `Options`, `SurfaceOptions`, `SurfaceTarget`,
  `Parameters`, `Shape`, `Paint`, `Stroke`, `Layer`, `Shadow`, `FontRef`,
  `FontData`, and `TextRun` values as direct construction.
- `scene(|s| ...)` produces the same `Scene` as direct `Scene::new()` mutation.
- Shape, stroke, layer, shadow, paint, and frame builders are inspectable before
  use.
- `RendererBuilder::options()` matches the options used by `build()`.
- `Frame::parameters()` matches the parameters used by `draw()`.
- `Surface::resize_metrics()` matches direct `Surface::resize()`.
- `Renderer::read()` matches direct `Renderer::read_headless()` diagnostics and
  output.
- Missing shadow paint produces a typed error or is prevented by type-state.
- Existing render tests continue to pass without changes to public behavior.
- Downstream `surgeist-dev` can use the DSL for at least one harness scene
  without changing render output intent.

Required smoke tests:

- Build a renderer through `renderer().build().await`.
- Create a headless surface through the surface DSL.
- Create a native surface through the window surface DSL when the `window`
  feature is enabled.
- Build a scene with fill, stroke, shadow, image, clip, layer, and text-run
  commands through the DSL.
- Submit through `renderer.frame(&mut surface).draw(&scene)`.

## Name List

Functions:

```text
renderer
scene
window
headless
font
font_data
point
size
rect
radii
round
circle
ellipse
path
transparent
rgb
rgba
linear
radial
sweep
image
stroke
dash
layer
shadow
text
```

Types:

```text
Aa
RendererBuilder
SurfaceTarget
WindowSurface
Headless
Frame
ShapeBuilder
PathBuilder
GradientBuilder
ImageBuilder
StrokeBuilder
DashBuilder
LayerBuilder
ShadowBuilder
TextBuilder
```

Existing render contracts remain public:

```text
Renderer
Surface
Scene
Paint
Image
Layer
Shape
Stroke
Shadow
Attachment
Options
SurfaceOptions
Parameters
Stats
FontRef
FontData
TextGlyph
TextPaint
TextRun
Error
Result
```
