# CSS And Style Integration Support Inventory

Date: 2026-07-07

## Purpose

This is the root-owned inventory for integrating the current `surgeist-css`
authored surface with the current `surgeist-style` receiving surface and the
downstream Surgeist crates.

It is not an implementation plan. It is the coordination spine for deriving
crate-local directives and root integration plans.

## Source Inputs

Primary style handoff package:

- `/Users/codex/Development/surgeist-style/plans/2026-07-07-style-root-handoff-notes.md`
- `/Users/codex/Development/surgeist-style/plans/2026-07-07-style-css-api-artifact.md`
- `/Users/codex/Development/surgeist-style/plans/2026-07-05-css-property-coverage-ledger.md`
- `/Users/codex/Development/surgeist-style/plans/2026-07-05-css-surface-style-ledger.md`
- `/Users/codex/Development/surgeist-style/plans/2026-07-05-css-surface-style-operations-sequence.md`

Current root integration baseline:

- `/Users/codex/Development/surgeist/src/adapters/css_style.rs`
- `/Users/codex/Development/surgeist/src/adapters/retained_style.rs`
- `/Users/codex/Development/surgeist/src/adapters/style_layout.rs`
- `/Users/codex/Development/surgeist/src/adapters/style_text.rs`

Important status note:

- The style handoff is newer than root's current style pointer at the time this
  inventory was written. Do not update root pointers from this inventory alone.
  Pointer updates still require the normal green-state submodule process.

## Boundary Rules

- `surgeist-css` owns strict authored CSS parsing, CSS syntax types, source
  locations, and parse diagnostics.
- root `surgeist` owns Surgeist-to-Surgeist lowering from CSS into style and
  from resolved style into downstream crate inputs.
- `surgeist-style` owns typed style receiving models, authored cascade data,
  selector/tree contracts, variable substitution, resolver behavior,
  diagnostics, invalidation facts, and computed style outputs.
- Downstream crates consume typed, resolved, crate-ready data. They must not
  consume raw CSS syntax.
- Style does not fetch imports, load fonts or images, schedule animations,
  render pixels, discover host facts, shape text, or compute layout.
- Root must reject unsupported integration inputs explicitly. It must not make
  downstream crates silently ignore CSS that the parser accepted.

## Inventory Status Labels

| Label | Meaning |
| --- | --- |
| `Ready for root lowering` | CSS and style have enough typed surface for root to implement lowering. |
| `Needs root policy` | CSS/style can model it, but root must decide strict behavior, diagnostics, or host facts. |
| `Needs downstream directive` | Style can expose data, but another crate needs a scoped plan to consume it. |
| `Deferred integration` | Accepted/authored syntax exists, but product behavior should wait for a later system. |
| `Root rejection initially` | Root should reject the parsed CSS surface until a future plan owns it. |

## Current Root Adapter Gap

Root's current CSS-to-style adapter is a narrow legacy mapper. It currently
handles a flat subset of style rules, simple selectors, and a small property
set through `style::Declarations`.

The new style surface expects root to lower into authored style APIs and rule
metadata:

- `AuthoredDeclaration`, `AuthoredDeclarations`, `AuthoredProperty`,
  `AuthoredValue`
- `CssWideKeyword`
- `CustomPropertyName`, `CustomPropertyValue`, `VariableDependentValue`
- `Sheet`, `Rule`, `RuleTarget`, `RulePrecedence`, `SourceOrder`,
  `LayerOrder`
- `Condition`, media/container query types, `RuleScope`
- `StyleBucket` and pseudo-element buckets
- `KeyframesRule` and animation/timing values

This means root needs a new authored-style lowering pass before downstream
crate directives can be fully implemented.

## Rule And At-Rule Inventory

| CSS surface | Style/root status | Other crate impact | Initial action |
| --- | --- | --- | --- |
| Style rules | Ready for root lowering | retained selector facts, style resolver | Replace legacy flat lowering with authored rule lowering. |
| `@layer` statements and blocks | Ready for root lowering | style cascade, invalidation | Root lowers encounter order into style layer APIs. |
| `@media` | Needs root policy | runtime/window host facts, style conditions | Define first supported media facts before lowering broadly. |
| `@container` | Needs root policy | layout/root container facts | Defer broad support until root can supply container facts. |
| `@scope` | Ready for root lowering with tests | retained/style selector facts | Lower only after selector facts are complete enough. |
| `@keyframes` | Ready for root lowering | runtime animation, render/text/layout consumers | Lower as symbolic style data first; sampling later. |
| `@font-face` | Deferred integration | text/root resource loading | Root/text directive needed before supporting. |
| `@import` | Deferred integration | root stylesheet loading/cache | Root owns import graph and fetch policy. |

## Selector And Tree Fact Inventory

| CSS/style capability | Root lowering | Retained/runtime need | Initial status |
| --- | --- | --- | --- |
| Tag, class, key selectors | Existing partial lowering | retained already has basic facts | Ready for root lowering refresh. |
| Compound and complex selectors | Needs new lowering | retained parent/sibling traversal | Needs retained fact audit. |
| Attribute selectors | Needs new lowering | retained must expose attributes consistently | Needs retained directive. |
| Structural selectors | Needs new lowering | retained traversal and projected/canonical policy | Needs retained directive. |
| `:is`, `:where`, `:not` | Needs new lowering | style specificity semantics | Ready after root selector lowering exists. |
| `:has` and relative selectors | Needs root policy | retained traversal cost/invalidation | Defer or support with conservative invalidation. |
| Runtime pseudo-classes | Needs root lowering | runtime/window state facts | Needs runtime/retained state directive. |
| `:root` and `:scope` | Needs new lowering | root scope anchors | Ready with scoped resolver tests. |
| Pseudo-elements | Needs root policy | retained/render/runtime materialization | Style buckets exist; product materialization deferred. |

Pseudo-element policy:

- Style buckets are acceptable because they do not require anonymous layout
  nodes.
- Root must decide which buckets can affect rendering in the first pass.
- `::before`, `::after`, and `::marker` require generated-content
  materialization before they produce output.
- `::selection` can be a style bucket for selection painting without becoming
  DSL-addressable.
- `::backdrop` likely waits for window/runtime policy.

## Declaration And Cascade Inventory

| Capability | Style status | Root/downstream need | Initial status |
| --- | --- | --- | --- |
| CSS-wide keywords | Style authored model exists | Root lowers keywords through authored declarations | Ready for root lowering. |
| `revert` and `revert-layer` | Style layer-aware paths exist | Root must decide origin support | `revert-layer` ready; `revert` needs origin policy. |
| Custom properties | Style store and resolution exist | Root lowers CSS authored tokens/references | Ready for root lowering. |
| `var(...)` ordinary values | Style variable-dependent path exists | Root builds typed fallback expressions | Ready for root lowering with tests. |
| `!important` | Not supported by style pass | Root strict policy | Root rejection initially. |
| Cascade origins | Not supported by style pass | Root policy | Author-origin only initially. |
| Source spans | Style has opaque `StyleSourceId` | Root source table | Needs root diagnostic plan. |

## Property Family Inventory

| Family | Style receiving status | Downstream owner pressure | Initial action |
| --- | --- | --- | --- |
| Display, box, position | Ready for root lowering | layout | Layout directive after root lowering inventory slice. |
| Sizing and spacing | Ready for root lowering | layout, text for font-relative units | Root/style/text policy for symbolic units before full layout lowering. |
| Flex and grid | Ready for root lowering | layout | Layout directive for exact supported values and rejections. |
| Alignment and writing mode | Ready for root lowering | layout, text | Layout/text directives for parity gaps. |
| Visibility/content-visibility | Ready for root lowering | layout, render/runtime | Needs root policy for hidden/skipped behavior. |
| Typography and font | Ready for root lowering | text | Text directive for normalized style consumption and font policy. |
| Inline metrics inputs | Style has font-size/line-height data | text/layout/root | Layout planning directive exists; text/root still need metric derivation policy. |
| Generated content/counters/lists | Style receiving model exists | retained/render/text/layout | Defer materialization; create separate directive. |
| Color and symbolic colors | Style receiving model exists | render/text/root theme facts | Render/text directives for final color realization. |
| Background layers | Style receiving model exists | render/resource loading | Render directive for gradients/images/repeat/position/size. |
| Borders, radii, outlines | Style receiving model exists | render/shape/layout for border widths | Render/shape/layout directives. |
| Shadows, opacity, filters | Style receiving model exists | render/compositor | Render directive. |
| Clip paths and masks | Style symbolic model exists | render/shape/resource loading | Render/shape directive; resource parts deferred. |
| Transforms | Style receiving model exists | render/layout hit testing/runtime | Render/runtime/window policy directive. |
| Cursor, pointer events, user-select | Style receiving model exists | window/runtime/retained | Runtime/window directive. |
| Transitions and animations | Style receiving model exists | runtime/render/text/layout | Runtime animation scheduler/sampler directive before rendering effects. |

## Downstream Crate Work Slices

### Root

Root needs the first implementation plan. It should:

- replace legacy `css_style` lowering with authored-style lowering
- preserve CSS source locations through a root source table
- lower rules, selectors, layers, scopes, conditions, declarations, variables,
  custom properties, keyframes, and style buckets
- reject unsupported integration surfaces explicitly
- keep root-owned imports, font-face, host facts, image/font loading, and
  animation scheduling out of style
- generate API artifacts only after pointer updates

### Retained

Retained needs a selector fact directive:

- attributes, classes, tags, keys, roles, states
- parent/child/sibling traversal facts
- canonical versus projected traversal policy
- runtime-state fact intake for pseudo-classes
- invalidation facts for selector matching

Retained should not materialize pseudo-elements as anonymous product nodes
without a separate root decision.

### Layout

Layout needs a directive for layout-facing resolved style consumption:

- display, sizing, spacing, flex, grid, alignment, writing mode, visibility,
  content-visibility, aspect ratio, order, overflow, and border widths
- explicit unsupported-value diagnostics from root if layout cannot consume a
  style-supported value yet
- inline metrics planning for line-height/baseline/forced line breaks

An inline metrics planning directive has been placed in the layout crate so
layout can design its own plan while root builds this inventory.

### Text

Text needs a style-facing directive:

- font family, weight, style, stretch, variant, feature settings
- font-size, line-height, letter spacing, whitespace/wrapping/breaking
- text decoration, transform, alignments, selection color
- deriving layout-ready inline metrics without depending on layout
- font resource policy with root/text ownership clearly separated

### Render

Render needs the broadest downstream paint directive:

- background layers, gradients, image layers, repeat, size, position, clip, and
  origin
- borders, radii, outlines, shadows, opacity
- filters, backdrop filters, masks, clip paths, transforms
- symbolic color realization, currentColor, system colors, color-mix, relative
  colors, and color-space decisions
- generated-content paint inputs after materialization exists

### Shape

Shape should receive a focused directive for geometry and stroke-backed paint:

- border radii and rounded boxes
- outline/border stroke geometry
- clip path basic shapes
- mask/shape geometry where render lowers symbolic style data into shape data
- dashed stroke behavior if exposed through a standards-based custom property
  or future DSL path

### Runtime

Runtime needs a directive for dynamic style facts:

- hover, focus, active, disabled/enabled, checked, modal/fullscreen/popover and
  related runtime pseudo-classes
- animation and transition clocks
- keyframe sampling orchestration
- invalidation/redraw scheduling
- selection state for `::selection`

Runtime should not own CSS or style parsing. It should consume root/style
runtime facts and produce invalidation/redraw pressure.

### Template

Template needs only authoring-surface awareness initially:

- classes, attributes, keys, roles, states, and custom properties in the IR
- no CSS-to-style lowering
- no style resolution
- enough source identity for root diagnostics and selector facts

### Test

Test should own cross-crate verification harnesses after root lowering exists:

- CSS fixture inputs for selectors, variables, layers, keyframes, and
  backgrounds
- root lowering golden tests
- resolved-style diagnostics tests
- integration/e2e app fixtures once runtime/render/layout/text are connected

## Suggested Directive Sequence

1. Root authored CSS-to-style lowering plan.
2. Retained selector fact and runtime-state fact directive.
3. Layout layout-facing property directive plus inline metrics plan.
4. Text style-consumption and metric-derivation directive.
5. Render paint/effects/background/transform directive.
6. Runtime pseudo-class and animation scheduler directive.
7. Shape geometry/stroke directive.
8. Template authoring-fact directive.
9. Test integration harness directive.

This sequence can run partially in parallel after root defines the lowering
contract, but root should not ask downstream crates to implement vague
"support all CSS" work. Each directive should name exact style outputs and the
crate-owned consuming API.

## Initial Root Decisions

These decisions are recommended for the first integration pass:

- Support author-origin cascade only.
- Reject `!important` if CSS parses it before style models it.
- Support `revert-layer`; treat full `revert` as unsupported until origin
  policy exists.
- Preserve custom properties and implement style-owned `var(...)` substitution
  through root lowering.
- Lower keyframes symbolically, but do not claim animations are visible until
  runtime sampling and render/text/layout interpolation exist.
- Lower pseudo-element buckets, but only materialize render output for buckets
  that have explicit downstream support.
- Reject `@font-face` and `@import` integration initially with root-owned
  diagnostics unless a root resource-loading plan lands first.
- Treat unsupported render/text/layout consumers as integration errors, not
  silent no-ops.

## Minimum Inventory Review Checklist

A clean-context reviewer should verify:

- The inventory is rooted in the style handoff package.
- It separates root lowering from style receiving models.
- It does not ask style to parse CSS or downstream crates to consume CSS.
- It names downstream crate responsibilities clearly enough to derive
  directives.
- It does not rely on stale old-plan assumptions.
- It preserves the project rule that root owns Surgeist-to-Surgeist adapters.
