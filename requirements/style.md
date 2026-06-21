# surgeist::style Requirements

`surgeist::style` is the typed rule, cascade, and resolved-style boundary for Surgeist. It owns style property vocabulary, selectors, rules, sheets, cascade order, inheritance, state matching, resolution contexts, resolved style snapshots, property metadata, invalidation classification, and animation-ready value descriptions.

The public API should be designed for use as `surgeist::style::*`. The module path supplies the layer name, so public type names should stay short and local: `Sheet`, `Rule`, `Selector`, `Compound`, `Part`, `Combinator`, `Declaration`, `Declarations`, `Property`, `Value`, `Length`, `Edges`, `Corners`, `Color`, `Shadow`, `Stroke`, `Resolved`, `Resolver`, `Context`, `Tree`, `Node`, `Traversal`, `StateFlag`, `Condition`, `Viewport`, `Container`, `Position`, `Nth`, `Metadata`, `Impact`, `Fingerprint`, `Version`, `Change`, `Invalidation`, `Error`, and `Result`.

The first contract is Rust-authored style. CSS parsing and hot loading should be specified separately after the typed style model is stable.

## Scope

This module owns:

- Typed style properties and values used by layout, text, shape, render, and animation.
- Sparse declaration sets for authored rules and local overrides.
- Resolved style snapshots with defaults and inherited values applied.
- Selectors for tag, class, id/key, retained state, child position, compound selectors, descendant selectors, child selectors, adjacent sibling selectors, and general sibling selectors.
- Conditional rule activation for viewport and container facts supplied by callers.
- Deterministic cascade order suitable for app UI instead of browser compatibility.
- Rule indexing by primary selector key for fast candidate lookup.
- Read-only tree access through a trait over retained snapshots or test fixtures.
- Resolution caching keyed by sheet version, tree version hints, node identity, state, viewport, container inputs, and declaration fingerprints.
- Invalidation classification for layout, paint, text, effect, and animation consumers.
- Property metadata that describes inheritance, default value, value category, animation support, and downstream impact.
- Headless tests for selector matching, cascade order, inheritance, invalidation, caching, and large-tree behavior.

## Boundary

`surgeist::style` consumes retained semantic facts through read-only adapters and produces resolved visual/layout/text facts for downstream layers.

Expected flow:

```text
retained snapshot + sheet + resolution context
  -> style resolver
  -> resolved style + invalidation facts
  -> layout / text / shape / render / animation consumers
```

Style values are logical, typed, and host-independent. Downstream layers decide how to translate resolved facts into layout constraints, shaped text, vector geometry, render scenes, accessibility projections, and animated frames.

## Dependencies

Expected direct dependencies:

```text
surgeist::style
  -> surgeist::retained
  -> surgeist::text
  -> peniko
```

`surgeist::style` may reuse public value types from lower foundation modules when those types are already the canonical representation. Examples:

- `Color` is style's canonical renderable color, implemented as `peniko::Color` or as a style newtype with explicit conversion into render paint.
- `surgeist::text` enums for resolved text wrapping, direction, alignment, line height, and decoration values when text already owns the precise backend contract.

The dependency direction should be:

```text
retained snapshots        -> style matching input
style resolved values     -> layout / text / shape / render / animation
animation sampled values  -> style resolution overlay
```

Animation depends on style property/value vocabulary. Style exposes the hook where sampled animated declarations can participate in final resolution.

## Module Layout

The implementation should be split into small internal modules with one public front door. Downstream users should import from `surgeist::style::*` unless a future API explicitly promotes a submodule.

Planned modules:

- `mod.rs`: public front door and curated re-exports.
- `error.rs`: `Error`, `ErrorCode`, `Result`, and diagnostic helpers.
- `value.rs`: `Value`, typed property value structs/enums, defaults, inheritance helpers, and validation.
- `property.rs`: `Property`, `Metadata`, `Impact`, value-category metadata, and animation support flags.
- `declaration.rs`: `Declaration`, `Declarations`, sparse set storage, merge logic, and typed accessors.
- `selector.rs`: `Selector`, `Compound`, `Part`, `Combinator`, `Position`, `Nth`, specificity-free matching primitives, and validation.
- `condition.rs`: `Condition`, `Viewport`, `Container`, and caller-supplied condition facts.
- `tree.rs`: `Tree`, `Node`, `StateFlag`, child/sibling/ancestor read adapters, and retained snapshot integration.
- `sheet.rs`: `Sheet`, `Rule`, `Version`, rule insertion, rule iteration, and rule indexes.
- `resolver.rs`: `Resolver`, `Context`, candidate collection, cascade, inheritance, overlays, and cache management.
- `invalidation.rs`: `Change`, `Invalidation`, property impact accumulation, and retained change classification.
- `tests.rs`: contract fixtures for selectors, cascade, resolution, invalidation, cache reuse, and large models.

Rules:

- Public re-exports should be intentional and stable.
- Storage details for indexes and caches should remain private.
- Validation should live near the type that owns the invariant.
- Add modules only for durable style boundaries.

## Naming

Public names are authored for the `surgeist::style` namespace:

- `Sheet` stores ordered style rules and rule indexes.
- `Rule` pairs one selector, one declaration set, a condition list, and source order.
- `Selector` describes a match expression over retained semantic facts.
- `Compound` describes one selector segment with tag, key/id, classes, states, attributes, and child-position constraints.
- `Part` describes one segment of a complex selector and its relationship to the previous segment.
- `Combinator` describes descendant, child, adjacent sibling, and general sibling relationships.
- `Declaration` stores one property/value pair.
- `Declarations` stores a sparse ordered set of declarations.
- `Property` identifies a typed style property.
- `Value` stores one typed property value.
- `Length` describes absolute, relative, intrinsic, fill, and auto style lengths.
- `Edges` stores four edge-scoped style lengths.
- `Corners` stores four corner-scoped style lengths.
- `Color` is the canonical style color value.
- `Shadow` describes one authored shadow.
- `Stroke` describes one authored stroke or border edge style.
- `Resolved` stores the complete resolved style for one node and context.
- `Resolver` resolves styles and owns reusable caches.
- `Context` supplies viewport, container, inherited parent style, local declarations, and animation overlays for one resolution pass.
- `Tree` is the read-only selector matching interface.
- `Node` is the read-only node fact view used by `Tree`.
- `Traversal` selects canonical or projected tree traversal for selector matching.
- `StateFlag` is the retained state fact used by selectors.
- `Condition` describes rule activation from viewport or container facts.
- `Viewport` describes available viewport facts.
- `Container` describes available container facts.
- `Position` describes child index and sibling count.
- `Nth` describes `an+b` child-position matching.
- `Metadata` describes property defaults, inheritance, value category, animation support, and downstream impact.
- `Impact` describes whether a property can affect layout, paint, text, effects, or animation.
- `Fingerprint` identifies declaration content for cache keys.
- `Version` identifies sheet revisions for cache keys.
- `Change` describes inputs that may affect resolved style.
- `Invalidation` describes downstream work caused by style changes.
- `Error` and `ErrorCode` are stable diagnostics.
- `Result<T>` is this module's result alias.

Use the module path for layer context. Prefer `surgeist::style::Sheet` over `surgeist::style::StyleSheet`.

## Style Values

`Value` is the typed storage for style declarations. It should be expressive enough for first-pass UI layout, paint, text, and geometry without becoming a parser AST.

Expected value categories:

```rust
pub enum Value {
    Keyword(Keyword),
    Number(f32),
    Length(Length),
    Size(Size),
    Edges(Edges),
    Color(Color),
    Corners(Corners),
    ShadowList(Vec<Shadow>),
    Stroke(Stroke),
    Text(TextValue),
    Transform(Transform),
    Visibility(Visibility),
}
```

Expected supporting value types:

- `Length`: absolute pixels, percent, fill, fit, min content, max content, auto.
- `Size`: width and height style values.
- `Edges`: top, right, bottom, and left style lengths for margin, padding, border widths, inset, and other edge-scoped properties.
- `Color`: style's canonical renderable color value.
- `Corners`: top-left, top-right, bottom-right, and bottom-left style lengths for radius-like properties.
- `Shadow`: offset, blur, spread, color, and inset flag.
- `Stroke`: color, width, side set, line style, dash facts, and alignment.
- `TextValue`: font family, font size, font weight, font style, line height, color, alignment, wrapping, whitespace, decoration, and selection colors.
- `Transform`: transform operations in logical style space.
- `Visibility`: visible, hidden, collapsed, or retained-only display intent.

Rules:

- Authored declarations may contain unresolved values such as percent, auto, fit, fill, and inherited.
- `Resolved` should preserve unresolved layout-relative values that only layout can solve.
- Style-owned `Edges` and `Corners` store unresolved `Length` values. Conversion into `shape::Insets`, `shape::Radii`, or other resolved geometry happens only after layout or a caller-owned resolution pass supplies the required dimensions.
- Values must be finite where numeric.
- Constructors should reject malformed values.
- Property accessors should be typed so callers do not downcast broad enums in normal use.
- Defaults should be centralized in `Metadata`.
- Inheritance should be property-specific.
- Local explicit declarations should use the same `Declarations` structure as rule declarations.

## Properties

`Property` is the stable identity of a style value. It should be narrow enough for invalidation and animation to reason about changes without string matching.

Expected first-pass property groups:

- Box: display, position, inset, width, height, min size, max size, margin, padding, overflow, z index.
- Flex/grid seeds: direction, flex direction, flex wrap, flex grow, flex shrink, flex basis, alignment, justify, gap, row gap, column gap.
- Paint: background, foreground, border color, border width, border style, radii, shadows, opacity, visibility.
- Text: font family, font size, font weight, font style, line height, text color, text alignment, text wrapping, whitespace, word break, overflow wrap, text overflow, decorations, selection color.
- Interaction visual state: cursor, pointer events, focus outline, selection paint.
- Effects: transform, transform origin, filter-ready effect slots.
- Animation metadata: transition property list, transition duration, transition delay, timing function, animation name references.

Rules:

- Each property must have a default value or a documented inherited source.
- Each property must declare whether it inherits by default.
- Each property must declare downstream impact through `Impact`.
- Each property must declare whether it is animatable and the interpolation category expected by `surgeist::animation`.
- Shorthand helpers may expand into longhand declarations, but storage should remain longhand enough for invalidation and animation.

## Declarations

`Declarations` is an ordered sparse set of property values.

```rust
pub struct Declaration {
    pub property: Property,
    pub value: Value,
}

pub struct Declarations {
    // private ordered sparse storage
}
```

Rules:

- Later declarations in the same set override earlier declarations for the same property.
- Merging declarations should be deterministic and stable.
- Shorthands should expand before storage.
- Typed builder methods should stay concise:

```rust
Declarations::new()
    .bg(color)
    .color(text_color)
    .padding(Edges::all(Length::px(8.0)))
    .radius(Corners::all(Length::px(6.0)))
```

- Verbose duplicate method families should be avoided. Prefer fluent chaining plus typed value constructors.
- `Declarations::fingerprint()` should return a stable content fingerprint for cache identity.

## Selectors

Selectors match retained facts through `Tree`.

Expected selectors:

```rust
pub enum Selector {
    Any,
    Tag(Tag),
    Class(Class),
    Key(Key),
    State(StateFlag),
    Attribute(AttributeSelector),
    Position(PositionSelector),
    Compound(Compound),
    Complex(Vec<Part>),
}

pub enum Combinator {
    Descendant,
    Child,
    Adjacent,
    Sibling,
}
```

Rules:

- Selectors should be deterministic and specificity-free for the first contract.
- Cascade order is source order after selector match and condition match.
- Compound selectors match one node.
- Complex selectors match from the target segment backward through parent and sibling relationships.
- Child position selectors use projected traversal when a projected tree is supplied.
- Attribute selectors should start with exact existence and exact equality.
- State selectors should map directly to retained state facts.
- Selector indexes should use the most selective available key: key/id, class, tag, then universal.

## Conditions

Conditions allow rules to activate from caller-supplied environmental facts.

```rust
pub enum Condition {
    Viewport(Viewport),
    Container(Container),
}
```

Rules:

- `Viewport` supports min/max width and min/max height.
- `Container` supports min/max width and min/max height.
- A `Rule` stores `conditions: Vec<Condition>`.
- A rule with no conditions is always active after its selector matches.
- A rule with conditions is active only when every condition matches.
- Container facts are supplied by the caller for the node being resolved.
- Missing container facts make container conditions fail.

## Tree View

`Tree` supplies read-only selector facts without creating a second style-owned tree.

```rust
pub trait Tree {
    type Id: Copy + Eq;

    fn version_hint(&self) -> Option<u64>;
    fn node(&self, id: Self::Id) -> Result<Node<'_, Self::Id>>;
    fn parent(&self, id: Self::Id, traversal: Traversal) -> Result<Option<Self::Id>>;
    fn children(&self, id: Self::Id, traversal: Traversal) -> Result<impl Iterator<Item = Self::Id> + '_>;
    fn previous_sibling(&self, id: Self::Id, traversal: Traversal) -> Result<Option<Self::Id>>;
}

pub enum Traversal {
    Canonical,
    Projected,
}
```

`Node` should expose tag, key/id, classes, attributes, text presence, role, and retained state facts through borrowed accessors.

Rules:

- `retained::Snapshot` should adapt into `Tree`.
- Tests should be able to provide a small fixture tree without constructing a full retained model.
- Selector matching should never mutate the tree.
- Selector matching should choose canonical or projected traversal explicitly through `Traversal`.
- `Traversal::Projected` uses `retained::ProjectionSlot::default(parent)` for child traversal.
- Named projection slots stay outside selector traversal until the style API explicitly models slot-scoped selector matching.
- Projected traversal should be used when style resolution is being performed for visual layout.
- Fallible retained traversal should surface as style diagnostics rather than forcing adapters to allocate slices or hide errors.

## Sheet

`Sheet` stores rules in deterministic order.

```rust
pub struct Rule {
    selector: Selector,
    declarations: Declarations,
    conditions: Vec<Condition>,
    order: u32,
}

pub struct Sheet {
    // private rules, indexes, version
}
```

Rules:

- `Sheet::new()` starts empty.
- Rule insertion increments `Version`.
- Extending a sheet preserves incoming rule order after existing rules.
- Rules can be queried by selector, class, tag, key/id, and condition presence.
- Indexes should be rebuilt or updated deterministically.
- Cloned sheets preserve the same `Version`.
- Mutating either the original or cloned sheet advances only that sheet's `Version`.
- Sheet equality should compare rules, not cache internals.

## Resolution

`Resolver` computes `Resolved` styles from a sheet and a tree.

```rust
pub struct Context<'a, T: Tree> {
    pub tree: &'a T,
    pub node: T::Id,
    pub traversal: Traversal,
    pub viewport: Viewport,
    pub container: Option<Container>,
    pub parent: Option<&'a Resolved>,
    pub local: Option<&'a Declarations>,
    pub animated: Option<&'a Declarations>,
}
```

Resolution order:

1. Start from property defaults.
2. Apply inherited values from parent where metadata says the property inherits.
3. Apply matching sheet rules in source order.
4. Apply local declarations supplied by the caller.
5. Apply animated declarations supplied by the caller.
6. Normalize dependent values that style can normalize without layout.

Rules:

- The cascade is deterministic and source-order based.
- `Context::new(tree, node)` should default to `Traversal::Projected`.
- Local declarations represent explicit app-authored overrides.
- Animated declarations represent sampled values for the current frame.
- Resolution should return stable diagnostics for unknown properties, invalid values, missing node ids, and invalid selector contexts.
- Resolved style should expose typed accessors for common consumers.
- Cache entries should be reusable when sheet version, `Tree::version_hint()`, node id, traversal, node state, viewport, container, and declaration fingerprints are unchanged.
- `Tree::version_hint() == None` disables cross-call resolution cache reuse unless the adapter supplies an equivalent stronger tree or node fingerprint through a future explicit cache key API.
- Cache invalidation should allow callers to clear all, clear by sheet version, or clear nodes touched by retained changes.

## Invalidation

`Invalidation` tells downstream layers what kind of work a style change implies.

```rust
pub struct Invalidation {
    pub layout: bool,
    pub paint: bool,
    pub text: bool,
    pub effect: bool,
    pub animation: bool,
}
```

Rules:

- Invalidation should be derived from changed properties and `Metadata`.
- Class, tag, key/id, attribute, state, parent, sibling, and child-position changes should map to selector rematch needs.
- Inherited property changes should mark descendants that depend on inherited values.
- Container fact changes should invalidate rules with matching container conditions.
- Viewport fact changes should invalidate rules with matching viewport conditions.
- Resolved-to-resolved comparison should classify downstream impact without re-running layout or render.
- Large trees should support targeted invalidation from retained `ChangeSet` data.

## Animation Readiness

Style defines the property and value vocabulary used by animation.

Rules:

- `Metadata` should identify animatable properties.
- `Value` should expose interpolation categories such as numeric, length, color, transform, radii, inset, shadow list, and discrete.
- `Resolved` should be comparable before and after an animation overlay.
- Transition properties should be representable as normal declarations.
- The animation runtime should be able to sample values into `Declarations` and pass them to `Resolver`.

## API Examples

Rust-authored sheet:

```rust
use surgeist::style as s;

let sheet = s::Sheet::new()
    .rule(
        s::Selector::class("primary")?,
        s::Declarations::new()
            .bg(s::color(0x2f6fedff))
            .color(s::color(0xffffffff))
            .radius(s::Corners::all(s::Length::px(8.0))),
    )
    .rule(
        s::Selector::compound()
            .tag("button")?
            .state(s::StateFlag::Hovered)
            .selector(),
        s::Declarations::new().shadow(s::Shadow::soft(0.18)),
    );
```

Resolution:

```rust
let mut resolver = s::Resolver::new(sheet);
let resolved = resolver.resolve(
    s::Context::new(snapshot.tree(), node)
        .viewport(s::Viewport::new(1280.0, 720.0))
        .local(&local_overrides),
)?;
```

Animation overlay:

```rust
let resolved = resolver.resolve(
    s::Context::new(tree, node)
        .parent(parent_style)
        .animated(animation.sample(node, now)),
)?;
```

## Testing

Required tests:

- Declaration merge order and typed accessor behavior.
- Property metadata defaults, inheritance flags, impact flags, and animation flags.
- Selector matching for tag, class, key/id, state, attributes, child position, compound, descendant, child, adjacent, and sibling selectors.
- Conditional rule activation from viewport and container facts.
- Cascade order with multiple matching rules.
- Parent inheritance.
- Local override precedence.
- Animation overlay precedence.
- Rule index candidate reduction.
- Cache reuse on repeated resolution.
- Cache invalidation after sheet edits.
- Invalidation classification for layout, paint, text, effect, and animation properties.
- Targeted invalidation from retained class, attribute, state, structure, projection, and text changes.
- Large tree behavior with at least 10,000 nodes and many shared classes.

Tests should remain headless and should not require a renderer, native window, CSS parser, app crate, or egui.
