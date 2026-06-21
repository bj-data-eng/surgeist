# surgeist::text Requirements

`surgeist::text` is the Parley-backed text boundary for Surgeist. It owns font discovery, font fallback, rich text shaping, line breaking, bidi layout, inline text style projection, cursor geometry, selection geometry, and editor-facing text movement primitives.

Source authoring ergonomics are specified separately in
[`text_source_composer.md`](text_source_composer.md). The source composer is a
range-capturing helper over `Source`, `Span`, `Range`, and `InlineBox`; it does
not replace the explicit layout API described here.

The public API should be designed for use as `surgeist::text::*`. The module path supplies the layer name, so public type names should stay short and local: `System`, `Builder`, `Source`, `Layout`, `Key`, `Style`, `Span`, `Range`, `InlineBox`, `Font`, `Brush`, `Selection`, `Cursor`, `Hit`, `Movement`, `Edit`, `Options`, `Metrics`, `Error`, and `Result`.

## Scope

This module owns:

- Parley integration.
- Font context creation and font fallback.
- Font family, weight, width, style, variation, and feature requests.
- Locale, direction, bidi, segmentation, normalization, and line breaking.
- Rich text span projection from Surgeist inline text data into Parley style runs.
- Inline box projection for embedded inline UI, icons, placeholders, and out-of-flow anchors.
- Text layout for paragraphs, labels, code, editable fields, and markdown-derived rich text.
- Glyph run, line, cluster, decoration, cursor, and selection geometry extraction.
- Text hit testing.
- Text movement primitives for keyboard and pointer selection behavior.
- AccessKit text range and selection conversion when accessibility support is enabled.
- Headless text layout tests without a renderer or native window.

This module describes text facts and text movement. It does not own GPU rendering, window creation, document tree identity, CSS parsing, command dispatch, app state, markdown parsing, or platform input capture.

## Dependencies

Expected direct dependencies:

```text
surgeist-text
  -> parley
  -> fontique
  -> peniko
  -> optional accesskit
```

The default `surgeist-text` crate must not depend on `surgeist-render`, Vello, `wgpu`, `surgeist-window`, document, widget, DSL, or app crates.

The dependency direction between text and render is one-way:

```text
surgeist-text  -> parley/fontique
surgeist-text  --optional render projection--> surgeist-render
```

This keeps text layout useful in tests, accessibility, selection, search, editing, markdown preview, and server-side preparation without requiring a GPU.

## Naming

Public names are authored for the `surgeist::text` namespace:

- `System` owns shared font and layout contexts.
- `Builder` builds one layout from text and spans.
- `Layout` is a shaped and line-broken text layout.
- `Key` describes cache identity for text, style, options, and font generation.
- `Style` is requested text style.
- `Span` is a range and style override.
- `Range` is a byte range in the source text.
- `Font` describes requested font matching properties.
- `Brush` describes text fill and decoration paint in text-space terms.
- `Metrics` reports layout size, lines, baselines, and overflow facts.
- `Line`, `Run`, `Cluster`, `Glyph`, and `InlineBox` expose prepared text facts.
- `Cursor` is a text insertion position with affinity.
- `Selection` is a text range with anchor/focus.
- `Hit` describes the result of text hit testing.
- `Movement` describes logical/visual text movement intent.
- `Edit` describes text mutation intent.
- `Options` configures wrapping, alignment, scale, locale, and direction.
- `Error` and `ErrorCode` are stable diagnostics.
- `Result<T>` is this module's result alias.

Avoid repeating `Text` in type names when the module path already supplies it. Prefer `surgeist::text::Layout` over `surgeist::text::TextLayout`.

## Source Model

Text layout operates on one UTF-8 string plus style spans over byte ranges.

```rust
pub struct Source {
    pub text: String,
    pub spans: Vec<Span>,
    pub boxes: Vec<InlineBox>,
}

pub struct Span {
    pub range: Range,
    pub style: Style,
}

pub struct Range {
    pub start: usize,
    pub end: usize,
}

pub struct InlineBox {
    pub id: Id,
    pub kind: InlineBoxKind,
    pub index: usize,
    pub size: Size,
}

pub enum InlineBoxKind {
    InFlow,
    OutOfFlow,
}
```

Rules:

- Ranges are byte ranges and must align to UTF-8 character boundaries.
- Invalid ranges produce errors.
- Overlapping spans are allowed only when the builder defines deterministic merge order.
- Source text must preserve user-authored whitespace unless explicit white-space style says otherwise.
- Inline box indices are byte offsets and must align to UTF-8 character boundaries.
- In-flow inline boxes participate in text layout; out-of-flow inline boxes produce positioned anchors without affecting text metrics.
- Sanitization of strings belongs at the document/input boundary, but this crate must reject malformed ranges and invalid text state.
- The crate should support cheap rebuilds when text or style changes are localized.

## Style Model

`Style` is the text subset of resolved Surgeist style.

```rust
pub struct Style {
    pub font: Font,
    pub size: f32,
    pub line_height: LineHeight,
    pub letter_spacing: f32,
    pub word_spacing: f32,
    pub brush: Brush,
    pub underline: Decoration,
    pub strikethrough: Decoration,
    pub locale: Option<Locale>,
    pub direction: Direction,
    pub white_space: WhiteSpace,
    pub word_break: WordBreak,
    pub wrap: Wrap,
    pub overflow_wrap: OverflowWrap,
}

pub enum LineHeight {
    MetricsRelative(f32),
    FontSizeRelative(f32),
    Absolute(f32),
}

pub enum WhiteSpace {
    Collapse,
    Preserve,
}

pub enum WordBreak {
    Normal,
    BreakAll,
    KeepAll,
}

pub enum Wrap {
    None,
    Word,
}

pub enum OverflowWrap {
    Normal,
    Anywhere,
    BreakWord,
}

pub enum Direction {
    Auto,
    LeftToRight,
    RightToLeft,
}
```

Rules:

- `Style` should map directly to Parley properties with minimal translation.
- Use CSS-inspired names only where they are already familiar and precise.
- Style resolution belongs above this crate. `surgeist::text` receives resolved text styles.
- Inline marks such as strong, emphasis, code, underline, and span become ordinary style spans before they reach this crate.
- `white_space`, `word_break`, `wrap`, and `overflow_wrap` should map to Parley's whitespace and line breaking controls.
- `LineHeight` should preserve Parley's distinction between font metrics relative, font size relative, and absolute line heights.
- `Direction` should map to Parley's base direction controls.
- The first text decoration API should match Parley's solid underline and strikethrough capabilities: enabled state, offset, size, and brush.
- Non-solid decoration styles such as wavy, dotted, dashed, and double require explicit Surgeist decoration geometry or render support and are not part of the first text crate contract unless promoted deliberately.
- The style model must support future markdown-derived rich text and editable rich text.

Current limitation:

- `WhiteSpace::Collapse` must fail with `UnsupportedFeature` until collapse can preserve authored source text, byte ranges, editing, and accessibility contracts.
- Explicit `Direction::LeftToRight` and `Direction::RightToLeft` must fail with `UnsupportedFeature` until the text backend can set base direction through the public layout path. `Direction::Auto` remains supported.

## Layout Options

`Options` configures paragraph-level layout policy around the resolved text styles.

```rust
pub struct Options {
    pub width: Option<f32>,
    pub scale: f32,
    pub alignment: Alignment,
    pub indent: Indent,
}

pub enum Alignment {
    Start,
    End,
    Left,
    Right,
    Center,
    Justify,
}

pub struct Indent {
    pub amount: f32,
    pub first_line: bool,
    pub each_line: bool,
    pub hanging: bool,
}
```

Rules:

- Alignment should map to Parley's `Alignment` and `AlignmentOptions`.
- Text indent should map to Parley's indent support and preserve first-line, each-line, hanging, and negative indent behavior where supported.
- Width changes may relayout cached text without rebuilding source/style runs when the source, style, and font generation are unchanged.

Current limitation:

- Indent combinations unsupported by the text backend must fail with `UnsupportedFeature`. In particular, `each_line` without `first_line` is not part of the first implementation contract.

## Core API

```rust
pub struct System {
    // private font and layout contexts
}

pub struct Builder<'a> {
    // private layout builder state
}

pub struct Layout {
    // private Parley layout wrapper
}

impl System {
    pub fn new(options: SystemOptions) -> Result<Self>;
    pub fn builder(&mut self, text: impl Into<String>) -> Builder<'_>;
    pub fn refresh_fonts(&mut self) -> Result<()>;
}

impl Builder<'_> {
    pub fn options(&mut self, options: Options) -> &mut Self;
    pub fn default_style(&mut self, style: Style) -> &mut Self;
    pub fn span(&mut self, range: Range, style: Style) -> &mut Self;
    pub fn inline_box(&mut self, box_: InlineBox) -> &mut Self;
    pub fn build(self) -> Result<Layout>;
}

impl Layout {
    pub fn metrics(&self) -> Metrics;
    pub fn lines(&self) -> Lines<'_>;
    pub fn glyph_runs(&self) -> GlyphRuns<'_>;
    pub fn hit(&self, point: Point) -> Hit;
    pub fn cursor(&self, cursor: Cursor) -> CursorGeometry;
    pub fn selection(&self, selection: Selection) -> SelectionGeometry;
}
```

`System` is reusable and cache-bearing. `Layout` is immutable after build. Editing creates new source text and then rebuilds or incrementally updates layout.

## Layout Caching

`surgeist::text` caches shaped layout facts. It does not cache glyph images, atlas entries, GPU resources, or renderer meshes.

```rust
pub struct Key {
    pub source: SourceKey,
    pub styles: StyleKey,
    pub options: OptionsKey,
    pub font_generation: u64,
}

pub struct SourceKey {
    pub id: Option<Id>,
    pub revision: u64,
    pub hash: u64,
}

pub struct StyleKey {
    pub revision: u64,
    pub hash: u64,
}
```

Rules:

- Cache identity must include source text, style spans, layout options, scale, locale, direction, and font generation.
- Font refresh invalidates affected layout cache entries through font generation changes.
- Width-only relayout may reuse Parley layout state when Parley supports it, but the first implementation may key by full options for simplicity.
- Cache statistics should report layout hits, layout misses, font refreshes, and invalidations.
- Glyph atlas caching belongs to Vello/glifo through `surgeist::render`, not this crate.
- Renderer-facing projection must not mutate the layout cache.

## Editing

`surgeist::text` should provide text movement and mutation primitives, not a full application editor.

```rust
pub enum Movement {
    PreviousCluster,
    NextCluster,
    PreviousWord,
    NextWord,
    LineStart,
    LineEnd,
    PreviousLine,
    NextLine,
    DocumentStart,
    DocumentEnd,
}

pub enum Edit {
    Insert { index: usize, text: String },
    Replace { range: Range, text: String },
    Delete { range: Range },
}
```

Rules:

- Movement must support logical and visual navigation where the distinction matters.
- Movement must support extending an existing selection.
- Insertion must target an explicit UTF-8 byte index so editor layers can apply cursor and IME text without appending by accident.
- Cursor affinity must be preserved for bidi and line boundary cases.
- Pointer hit testing produces cursor or selection intent, not app commands.
- This crate should expose enough geometry for a higher editor layer to draw cursor, selection, spelling marks, find matches, and IME composition.
- Clipboard, undo history, command routing, and document mutation ownership belong above this crate.

## Layout Output

`Layout` must expose text facts that can be projected into renderer-ready runs without exposing the whole Parley API as the Surgeist API.

```rust
pub struct Metrics {
    pub size: Size,
    pub line_count: usize,
    pub first_baseline: Option<f32>,
    pub last_baseline: Option<f32>,
    pub overflow: bool,
}

pub struct Glyph {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub advance: f32,
    pub range: Range,
}

pub struct Run<'a> {
    pub font: FontRef<'a>,
    pub style: Style,
    pub glyphs: &'a [Glyph],
}

pub struct PositionedInlineBox {
    pub id: Id,
    pub kind: InlineBoxKind,
    pub rect: Rect,
    pub index: usize,
}

pub enum Hit {
    Text(Cursor),
    InlineBox(Id),
    None,
}

pub struct DecorationRun {
    pub rect: Rect,
    pub brush: Brush,
    pub kind: DecorationKind,
}
```

Rules:

- Expose enough glyph run data for a render bridge to draw text without re-shaping.
- Expose positioned inline boxes so higher layers can place embedded UI and out-of-flow anchors consistently with text layout.
- Expose enough line and cluster data for selection, hit testing, accessibility, and editing.
- Do not expose Vello types from this crate.
- Do not expose glifo cache keys from this crate.
- Parley escape hatches may be available under an explicit advanced API, but the public front door should be Surgeist-shaped.

## Render Projection

The text crate may provide an optional projection into `surgeist::render` text-run input. This projection is a convenience boundary, not the owner of rendering.

Rules:

- The default `surgeist::text` crate must build without `surgeist::render`, Vello, glifo, `wgpu`, or a native window.
- A `render` feature or separate bridge crate may convert `Layout` glyph runs into `surgeist::render::TextRun` values.
- Projection must preserve font reference, glyph id, glyph position, font size, paint, and transform facts needed by Vello/glifo.
- Projection must not perform shaping, line breaking, selection calculation, or glyph rasterization.
- Glyph cache keys, atlas entries, and upload state remain renderer internals.

## Inline Boxes

Parley supports inline boxes as first-class layout items. `surgeist::text` should preserve that capability as a low-level text layout primitive.

Rules:

- Inline boxes are identified by stable caller-provided ids.
- `InlineBoxKind::InFlow` participates in line breaking and reserves inline space.
- `InlineBoxKind::OutOfFlow` receives a text-relative position without changing text layout metrics.
- Custom out-of-flow behavior may be exposed later if a higher layout layer needs float-like control.
- Inline boxes are layout facts only; rendering or updating embedded UI belongs above this crate.
- Hit testing should report whether a point maps to text, an inline box, or neither.

## Accessibility

Rules:

- Convert cursor and selection to AccessKit text positions/ranges when the `accessibility` feature is enabled.
- Expose text bounds, line bounds, character ranges, and selection facts needed by the accessibility tree.
- Accessibility conversion must not require a renderer.
- Accessibility conversion must preserve source byte ranges and user-visible text.

## Errors

```rust
pub type Result<T> = std::result::Result<T, Error>;

pub struct Error {
    pub code: ErrorCode,
    pub message: String,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

pub enum ErrorCode {
    FontSystemUnavailable,
    FontLoadFailed,
    InvalidRange,
    InvalidStyle,
    LayoutFailed,
    HitTestFailed,
    UnsupportedFeature,
}
```

Error codes must remain stable. Display messages may improve over time.

## Tests

Required contract tests:

- UTF-8 range validation rejects invalid span boundaries.
- Default style maps to Parley without losing font family, size, weight, style, width, features, variations, line height, letter spacing, word spacing, locale, direction, or decorations.
- White-space collapse/preserve, word break, wrap mode, and overflow wrap map to Parley's line breaking controls.
- Text indent maps to Parley's indent controls for first-line, each-line, hanging, and negative indent cases.
- Span merge order is deterministic.
- Inline box indices reject invalid UTF-8 boundaries.
- In-flow inline boxes affect line metrics and out-of-flow inline boxes preserve line metrics while producing positioned anchors.
- Layout metrics report size, line count, baselines, and overflow.
- Hit testing maps points to stable hit results.
- Hit testing can distinguish text hits, inline box hits, and empty-space hits.
- Cursor geometry is produced for line starts, line ends, bidi boundaries, and empty text.
- Selection geometry is produced for single-line, multi-line, and bidi selections.
- Movement handles cluster, word, line, and document navigation.
- Accessibility conversion preserves positions and selection ranges.
- Layout can be built without a renderer or native window.

Required smoke tests:

- Build one plain text layout.
- Build one mixed-style rich text layout.
- Build one wrapped paragraph layout.
- Build one layout with an inline box.
- Hit test a point into a text, inline box, or empty-space result.
- Produce selection geometry.
- Produce glyph runs that a renderer bridge can consume.

## First Milestone

Create a minimal Parley-backed implementation that:

1. Builds a reusable `System`.
2. Builds a `Layout` from plain text and resolved style spans.
3. Exposes metrics, lines, glyph runs, cursor geometry, and selection geometry.
4. Supports basic movement primitives.
5. Includes no dependency on Vello, `wgpu`, `winit`, or higher Surgeist UI crates.
6. Provides tests for invalid ranges, wrapping, hit testing, and glyph run extraction.
