# surgeist::text Source Composer Requirements

`surgeist::text` should provide a small source composer for authoring rich text
sources without manually calculating byte ranges. The composer is an authoring
surface over `Source`, `Span`, `Range`, and `InlineBox`; it does not replace the
layout `System`, `Builder`, or `Layout`.

The public API is designed for the `surgeist::text` namespace. Names stay short
because the module path supplies the layer: `source`, `compose`, `Composer`,
`Mark`, `Source`, `Span`, `Range`, `Style`, `InlineBox`, `InlineBoxKind`, `Id`,
`Size`, `Error`, and `Result`. `Mark` means a captured source range token, not
a semantic rich-text mark.

## Purpose

Manual byte ranges are the main ergonomic and correctness hazard in text
authoring. The composer captures ranges from inserted UTF-8 text, attaches
resolved styles to those ranges, inserts inline boxes at valid source positions,
and returns an ordinary inspectable `Source`.

This keeps low-level text layout explicit:

```rust
let source = text::source(|t| {
    t.push("Parley-backed text through Surgeist render. ");
    t.with(strong, |t| {
        t.push("Strong spans");
    });
    t.push(", ");
    t.with(color, |t| {
        t.push("color spans");
    });
});

let layout = system
    .layout(source, base_style, options)?;
```

The composer should make the common path humane while preserving the existing
`Source` model as the single interchange format.

## Scope

This module owns:

- Range capture while appending UTF-8 text.
- Style span insertion for text added inside scoped closures.
- Explicit empty spans when a caller deliberately wants one.
- Inline box insertion at the current source index.
- Source identity and revision assignment.
- Strict construction of `Source` values that existing layout validation can
  consume.

This module does not own markdown parsing, HTML parsing, CSS/style resolution,
semantic marks such as strong or emphasis, layout, rendering, editing history,
document identity, command routing, or app state.

## Public Surface

The composer is exported from `surgeist::text`:

```rust
pub fn source(children: impl FnOnce(&mut Composer)) -> Source;
pub fn compose() -> Composer;

pub struct Composer {
    // private Source
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Mark {
    // private Range
}
```

`lib.rs` should re-export `source`, `compose`, `Composer`, and `Mark`.

Rules:

- `source(|t| ...)` creates a new `Composer`, runs the closure, and returns the
  composed `Source`.
- `compose()` creates a reusable `Composer` for callers who need conditional
  construction before calling `finish`.
- `Composer` owns one `Source` internally.
- `Mark` is a lightweight captured range token. It does not own text or style.
- `Mark` keeps its range private so `span(Mark, Style)` can trust that the range
  was produced by the composer.
- The composer API is infallible where it only appends valid Rust strings and
  inserts boxes at the current source boundary. Existing layout validation still
  validates style facts and backend limitations.

## Composer Methods

Required methods:

```rust
impl Composer {
    pub fn new() -> Self;
    pub fn identity(&mut self, id: Id, revision: u64) -> &mut Self;
    pub fn revision(&mut self, revision: u64) -> &mut Self;
    pub fn push(&mut self, text: impl AsRef<str>) -> Mark;
    pub fn mark(&self) -> Mark;
    pub fn span(&mut self, mark: Mark, style: Style) -> &mut Self;
    pub fn with(&mut self, style: Style, children: impl FnOnce(&mut Composer)) -> Mark;
    pub fn box_(&mut self, id: Id, kind: InlineBoxKind, size: Size) -> &mut Self;
    pub fn try_span(&mut self, range: Range, style: Style) -> Result<&mut Self>;
    pub fn try_inline_box(&mut self, box_: InlineBox) -> Result<&mut Self>;
    pub fn source(&self) -> &Source;
    pub fn finish(self) -> Source;
}

impl Default for Composer;

impl Mark {
    pub fn range(self) -> Range;
    pub fn is_empty(self) -> bool;
}
```

Rules:

- `push` appends exactly the provided string and returns the range that was
  appended.
- `mark` returns an empty mark at the current source end.
- `span(mark, style)` appends a `Span` for `mark.range`.
- `with(style, children)` records the source length and current span count
  before running `children`, records the source length after the closure, inserts
  one span for that exact range at the recorded span index, and returns the
  captured mark.
- `with` preserves nested override order by inserting the outer span before any
  spans authored inside the closure. Nested spans remain later in the span list
  and therefore win under the existing last-matching-span behavior.
- `box_` inserts an `InlineBox` at the current source end.
- `try_span` and `try_inline_box` preserve explicit index/range escape hatches
  with typed errors.
- `source()` exposes the in-progress source for inspection.
- `finish()` returns the internal `Source` without additional transformation.
- `identity(id, revision)` sets source identity and revision.
- `revision(revision)` updates only the source revision.

## Range Semantics

Composer-generated ranges are byte ranges over the final `Source::text`.

Rules:

- Ranges produced by `push`, `mark`, and `with` are always valid UTF-8
  boundaries.
- Empty ranges are allowed. They are useful for cursor-style style anchors,
  future insertion policy, and tests.
- `with` over a closure that appends no text produces an empty mark and may still
  append an empty span.
- The composer must not trim, normalize, collapse, sanitize, or otherwise alter
  authored string content.
- Sanitization and semantic parsing remain above `surgeist::text`; this crate
  preserves strings and validates structural correctness.

## Style Semantics

The composer receives resolved `Style` values.

Rules:

- Composer methods do not know about strong, emphasis, code, links, markdown,
  HTML tags, classes, or CSS selectors.
- Callers may define their own style helpers outside this crate and pass the
  resulting `Style` values into `with` or `span`.
- Overlapping spans are allowed and retain declaration order.
- The existing layout builder remains responsible for mapping styles into
  Parley and reporting unsupported style features.

## Inline Boxes

Inline boxes can be inserted tersely at the current source position:

```rust
let source = text::source(|t| {
    t.push("before ");
    t.box_(icon_id, text::InlineBoxKind::InFlow, text::Size::new(16.0, 16.0));
    t.push(" after");
});
```

Rules:

- `box_` always uses the current source end as the inline box index.
- `try_inline_box` is the explicit index-based insertion path.
- There is no infallible `inline_box(InlineBox)` composer method in the first
  milestone. Callers who want arbitrary indices should use `try_inline_box`, and
  callers who want current-position insertion should use `box_`.

## Fallible Escape Hatches

The primary composer path should be infallible. Explicit index-based operations
are fallible:

```rust
impl Composer {
    pub fn try_inline_box(&mut self, box_: InlineBox) -> Result<&mut Self>;
    pub fn try_span(&mut self, range: Range, style: Style) -> Result<&mut Self>;
}
```

Rules:

- `try_span` validates the supplied range against the current source text.
- `try_inline_box` validates the supplied index against the current source text.
- Infallible `span(Mark, Style)` does not need validation because `Mark` is
  composer-produced.
- `try_span` and `try_inline_box` return `ErrorCode::InvalidRange` for invalid
  byte ranges or invalid UTF-8 boundaries.
- Shared validation helpers may move from `system.rs` into `range.rs` or
  `source.rs` so the composer can validate explicit ranges without depending on
  `System`.

## Integration With Layout

The existing layout APIs remain the explicit layout boundary:

```rust
let source = text::source(|t| {
    t.push("hello");
});

let layout = system.layout(source, Style::default(), Options::default())?;
```

Rules:

- `System::layout(Source, Style, Options)` remains the direct path from source
  to layout.
- `System::builder(text)` remains available for callers who already have plain
  text and explicit ranges.
- The composer should not require a `System` or access to Parley contexts.
- The composer should not cache layout data.

## Testing Contract

Required tests:

- `push` appends text unchanged and returns the appended byte range.
- `source(|t| ...)` returns the same `Source` as manual `Source::push` and
  `Source::span` calls.
- `with` captures only the text appended inside its closure.
- Nested `with` calls put outer spans before inner spans so inner spans win when
  ranges overlap.
- `with` over no appended text produces a valid empty range.
- UTF-8 multi-byte text produces correct byte ranges.
- `box_` inserts an inline box at the current source end.
- `try_span` rejects invalid ranges and non-boundary UTF-8 ranges.
- `try_inline_box` rejects invalid indices and non-boundary UTF-8 indices.
- `identity` and `revision` update the resulting `Source` identity fields.
- A composed source builds successfully through `System::layout` in a smoke
  test.

## First Milestone

Implement the composer in a new `composer.rs` module inside `surgeist-text`.
Export it from `lib.rs`, add focused tests beside the existing text tests, and
do not change existing `Source`, `System`, `Builder`, or `Layout` behavior.
