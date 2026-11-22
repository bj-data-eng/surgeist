# Explanation

## Animation ownership

This crate owns the standards-oriented animation engine layer for Surgeist:
easing evaluation, transition timing, keyframe timing, iteration, direction,
fill-mode, play-state, interpolation contracts, sampled animation values, and
animation-specific diagnostics.

- `surgeist-css` owns authored CSS animation and transition syntax and values;
  CSSOM owns live authored editing.
- `surgeist-animation` owns executable animation timing, time-space conversions
  and sampling contracts.
- `surgeist-runtime` owns clocks, scheduling, lifecycle, and invalidation.
- Root `surgeist` owns cross-crate lowering and integration.
- `surgeist-render`, `surgeist-layout`, and `surgeist-text` consume sampled
  values through root-owned integration.

The crate and facade now share one source repository and Cargo workspace.
Cross-crate composition and generated API audits remain root-owned; their current
behavior and coverage come from root source and verification evidence.

Computed font palettes follow the same boundary. Animation retains opaque
computed operand identities and returns flat Oklab mix plans. Style keeps their
keyword/name/mix mapping, determines context-dependent equivalence and resolves
palette entries against the selected font and rules. Nested sampled mixes keep
their operation boundaries; they do not introduce Color or font-resource
dependencies. See [computed font properties](reference.md#computed-font-properties).

## Why inputs are normalized

Separating authored style from executable timing lets callers resolve property
support, computed endpoints, keyframe lookup, and list pairing before sampling.
Runtime resolves clocks and track start times into effective elapsed time. The
animation crate can then sample explicit inputs without owning a scheduler or
CSS parser. Root owns the adapters between these boundaries.

The separate model-time boundary accepts signed coordinates and immutable
timeline/animation snapshots. Callers supply document activity and frame time;
Animation owns fixed-origin arithmetic and coordinate transformations. Captured
snapshots keep queue-time conversion inputs coherent while callers later change
timeline or playback state. Play-control lifecycle and effect sampling consume
these values separately. See [model time](reference.md#model-time-and-timeline-conversions).

`EffectTiming` consumes the associated animation's optional signed local time and
observations of playback, timeline monotonicity and replace state. Its phase,
fill and iteration calculations share one kernel with normalized CSS elapsed
sampling, using milliseconds for the model and seconds for existing track
inputs. Relevance is an Animation-owned predicate over those supplied facts;
consumers own target enumeration, lifecycle changes and invalidation. See
[effect timing](reference.md#effect-timing-and-relevance).

`AnimationRecords` owns the pure association graph, creation order and complete
playback transitions. Commands consume explicit timeline/effect facts, prepare
only affected records and commit their state together with ordered readiness,
completion and playback-event requests. Runtime executes those requests at the
prescribed deferred processing points and owns publication and dispatch. Ready
task generations reject stale work; finish-notification tokens instead recheck
live state and the current completion identity. Immutable `PlaybackCoordinates`
still provide independent checked coordinate observations. See
[playback ownership](reference.md#playback-ownership-and-native-controls).

Native keyframe inputs have a separate normalization boundary in
`keyframe_input`. It owns offset validation, property-track merging, list
repetition, and missing-offset calculation. Callers supply parsed property
payloads and easing validation results, so this boundary retains authored
meaning and source provenance. Its opaque result certifies normalized offsets
and metadata. Style supplies target-context computed property values. The
[reference](reference.md#native-keyframe-normalization) describes the phases
and failure semantics.

`keyframe::ComputedKeyframes` consumes the next phase: caller-computed values
with required computed offsets and preserved specified metadata. It filters each
longhand, preserves overlapping frame order, inserts native underlying-value
endpoints when required, and applies endpoint composition before interval easing
and interpolation. CSS supplies its separately assembled endpoints. The shared
pure composition boundary accepts selected property operations and validates
effect ordering over supplied snapshots. It folds each effect against the
previous output, without repeating endpoint composition. State publication stays
caller-owned. Stack playback facts come directly from checked records rather
than another supplied mutable play state. Computed effect bundles retain their
associated effect identity and setup revision, so reassociation or setup edits
cannot silently reuse stale frames. See [computed effects](reference.md#computed-keyframe-effects)
for native versus CSS timing adapters and atomic failure behavior.

The same immutable snapshot supplies replacement decisions. Exact target and
property coverage, playback state and timeline monotonicity determine eligibility;
persisted animations may cover lower animations while remaining protected from
removal. Runtime publishes the decisions and queues playback events from captured
coordinates. CSS owners supply class-specific order and applicability facts.
See [effect stacks](reference.md#effect-stacks-and-replacement).

CSS animation-list reconciliation uses `css_animation`. Style supplies the
resolved effective list, rule matches, used timing and applicability. Animation
matches occurrences from last to first, retaining native identity and captured
playback coordinates across rule or timing edits. It returns timing comparisons
and start/retain/stop plans. Runtime applies playback requests and publishes the
result coherently; lifecycle event dispatch remains a separate Runtime consumer.
See [CSS animation reconciliation](reference.md#css-animation-reconciliation).

CSS transition lifecycle decisions use a separate immutable snapshot boundary in
`transition`. Style provides atomic endpoint and matching facts; Animation
promotes due records and decides creation, cancellation, and reversal using the
same supplied property policy as current-value sampling. The resulting candidate
separates completion promotion from subsequent style outcomes so Runtime can
publish state before dispatching events. Keeping these decisions independent of
DOM mutation and clocks permits replay and failure without partial publication.
See [transition decisions](reference.md#css-transition-lifecycle-decisions).

## Values and failure semantics

Unrestricted finite percentages and eased progress preserve values outside the
unit interval. A resolved color pair keeps the method selected by the consumer
and delegates conversion, missing components, hue and premultiplication to
Color. Explicit premultiplied operands for shadow and filter composition use
their component algebra. Consumers retain responsibility for contextual
resolution, display gamut mapping, and final property clamping.

Unsupported interpolation is a permanent capability outcome; arithmetic failure
is local to a sample and can recover at a later endpoint. Next-frame hints also
account for play state, so paused samples remain stable until external input.
See the [reference](reference.md) for the exact contracts and limitations.

Computed property records combine as a unit: incompatible or differing discrete
components switch the complete record. Numeric components retain canonical
dimensions and independent percentage terms. Source/context identities preserve
unresolved computed calculations in flat symbolic operation plans; the caller
retains and evaluates the original calculations. This keeps authored CSS and
contextual resolution outside Animation while allowing later effect stacks to
add or accumulate underlying values before interpolating their endpoints.

Resolved transforms are executable Animation values, separate from CSS's
authored functions and Render's scene geometry. Keeping the typed list until
sampling preserves matching prefixes and rotation turns; flattening it eagerly
would lose that meaning. Consumers supply resolved CSS-pixel lengths and update
them when reference-box facts change. Animation owns interpolation and checked
2D/3D algebra and list accumulation; consumers own transform-origin placement
and singular visibility.
This requires no dependency on Layout, Shape, Paint or Render.

Computed basic shapes retain their numeric components and semantic reference
box instead of flattening into geometry. This permits interpolation of symbolic
length-percentage values while preserving the caller's source/context identity.
Compatibility is decided for the whole pair before sampling any component, so a
late radius keyword or polygon metadata conflict cannot publish a partially
interpolated shape or be hidden by an earlier arithmetic failure. Used radii,
overlap adjustment and polygon geometry stay with consumers. See the
[basic-shape contract](reference.md#computed-basic-shapes).

Native shape/path commands preserve command identity, absolute/relative forms,
explicit control anchors and implicit smooth controls. Sampling needs no path
execution or box geometry: it combines corresponding numeric operands after
whole-list compatibility. This also keeps authored CSS/SVG parsing with CSS and
resolved geometry with its consuming owner. See the
[command contract](reference.md#native-shape-and-path-commands).

Computed clip paths retain their complete branch above the same shape kernel.
Opaque source identities are guarded without URL parsing, resource lookup or
geometry execution. Pairing a shape with a different branch switches the whole
value; pairing shapes preserves their shared compatibility and typed errors.
Property names do not select interpolation policy: Masking's numeric-looking
mask-border properties remain discrete because their property tables require it.
See [computed Masking values](reference.md#computed-masking-values).

Box and text shadows share one normalized list algebra above their supplied
resolved geometry and premultiplied colors. Input guards remain distinct from
sampled values, so easing extrapolation does not trigger authored range checks.
The selected text shape permits inset and spread; a filter drop shadow's standard
deviation belongs to its distinct function shape. Shared box/text algebra does
not confer filter-list composition semantics or own layout, painting, currentcolor
resolution, color conversion or invalidation.

Filter lists preserve function identities until interpolation, so Animation can
preflight the entire list before numeric work and initialize unmatched tails with
the correct identities. Filter images reuse numerical list interpolation but
route incompatibility to the caller's generic image operation, while the filter
property uses whole-value discrete switching. Immutable source tokens distinguish
verified used image/context identity from authored URL spelling. The renderer
still owns image domains, blur edge handling and pixel execution. Neither native
list sampling nor generic-image delegation claims composed Filter Effects
rendering conformance.

Background and mask positions share normalized computed positions. Their sizes
share a guarded two-axis or Contain/Cover record. Matching size keywords remain
constant while numeric axes combine, so they remain separate from ordinary
discrete components. Both properties reuse the generic repeatable-list owner
for checked LCM pairing, global compatibility and atomic results. This keeps
contextual image/box resolution with callers and geometry consumers. See the
[layer contract](reference.md#background-and-mask-layer-positions-and-sizes).

## App-native motion

This remains distinct from app-native motion work. A future `surgeist-motion`
crate could own higher-level springs, gestures, layout transitions, presence,
and scroll-linked motion if those boundaries become concrete. That possibility
does not add those responsibilities to this crate.
