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

Native keyframe inputs have a separate normalization boundary in
`keyframe_input`. It owns offset validation, property-track merging, list
repetition, and missing-offset calculation. Callers supply parsed property
payloads and easing validation results, so this boundary retains authored
meaning and source provenance. Its opaque result certifies normalized offsets
and metadata. Style supplies target-context computed property values. The
[reference](reference.md#native-keyframe-normalization) describes the phases
and failure semantics.

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
unit interval. Explicit premultiplied color spaces avoid ambiguity about the
representation being interpolated. Consumers retain responsibility for conversion,
gamut mapping, and final property clamping.

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

Resolved 2D transforms are executable Animation values, separate from CSS's
authored functions and Render's scene geometry. Keeping the typed list until
sampling preserves matching prefixes and rotation turns; flattening it eagerly
would lose that meaning. Consumers supply resolved CSS-pixel lengths and update
them when reference-box facts change. Animation owns interpolation and checked
affine algebra; consumers own transform-origin placement and singular visibility.
This requires no dependency on Layout, Shape, Paint or Render.

## App-native motion

This remains distinct from app-native motion work. A future `surgeist-motion`
crate could own higher-level springs, gestures, layout transitions, presence,
and scroll-linked motion if those boundaries become concrete. That possibility
does not add those responsibilities to this crate.
