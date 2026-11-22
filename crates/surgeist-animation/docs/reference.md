# Reference

## Package and public interface

[Cargo.toml](../Cargo.toml) declares `surgeist-animation` 0.1.0, Rust edition 2024,
and the library import name `surgeist_animation`. It declares no runtime
dependencies, features, or explicit minimum Rust version. The dev-only
`stats_alloc` dependency instruments allocation measurements.

[src/lib.rs](../src/lib.rs) declares the public modules `computed`,
`diagnostics`, `easing`,
`interpolation`, `keyframe`, `keyframe_input`, `progress`, `sample`, `time`,
`timeline`, `timing`, and `transition`, and the constant `CRATE_NAME`. Each public
type has one path through its owning module, such as
`surgeist_animation::easing::Easing`. The crate forbids unsafe code. Tests live in
source modules and the external consumer targets under
[tests/](../tests/); crate-level examples are doctests.

## Model time and timeline conversions

`time::{AnimationTime, TimelineTime, OriginRelativeTime}` are distinct signed,
finite millisecond coordinates. Their constructors reject NaN and infinities;
`Option<coordinate>` exposes unresolved time separately from zero. `PlaybackRate`
accepts every finite multiplier, including negative values and zero. The current
representation uses `f64` milliseconds and distinguishes 0.001 ms from 0.0 ms in
the focused checks. This meets the selected specification's recommended input
precision at that boundary; it does not promise microsecond spacing at every
magnitude or require a fixed internal representation.

`timeline::DocumentTimeline` retains a caller-assigned identity and fixed
origin-time offset. Each `update(DocumentClock)` receives document activity and,
when active, an origin-relative frame timestamp. Current time is `now - origin`;
unestablished origins and inactive documents produce unresolved time. Active
reports cannot decrease, including across inactivity. Rejected reports preserve
the previous state. Reading `snapshot()` never advances time. On restoration,
the next update uses the supplied current timestamp without subtracting inactive
time or changing origins. The caller owns clock acquisition, document lifetime,
default-timeline uniqueness and frame/event ordering.

`TimelineSnapshot` captures current time, identity, monotonicity and optional
fixed-offset origin conversion. External producers may supply nonmonotonic
timelines with no origin conversion; time is not assumed to follow a wall clock.
External producers own their declared monotonicity across reports.

The conversion functions consume explicit coordinates and immutable snapshots:

| Function | Calculation and unresolved cases |
| --- | --- |
| `timeline_time_to_origin_relative_time` | Adds the timeline's own origin offset. Unresolved input, inactivity or absent conversion yields unresolved. |
| `animation_time_to_timeline_time` | Computes `time / rate + start`. Unresolved or infinite input, zero rate or unresolved start yields unresolved. |
| `animation_time_to_origin_relative_time` | Composes the two operations through the animation's captured associated timeline. Missing association also yields unresolved. |
| `timeline_time_to_animation_time` | Computes `(time - start) * rate` for supplied coordinates. Unresolved input or start yields unresolved. This arithmetic does not resolve hold time or play state. |

`AnimationTimeInput::Infinite` represents the extended algorithm input, such as
an infinite associated-effect end, without admitting nonfinite model coordinates.
`AnimationSnapshot` captures start time, playback rate and the associated timeline;
later timeline/rate changes cannot alter a previously captured conversion.
For animation time 30, rate 2, start 100 and document origin 40, the intermediate
timeline time is 115 and the origin-relative result is 155. An already supplied
timeline coordinate 30 converts directly to 70 without applying animation state.

These operations implement the adopted native repairs in
[#604](https://github.com/bj-data-eng/surgeist/issues/604) and
[#605](https://github.com/bj-data-eng/surgeist/issues/605): restored document time
uses current `now`, and failed animation conversion propagates unresolved rather
than returning the original input. This is a bounded repair of the selected
source's converter signature and return, not a claim that its text is corrected.
The [module example](../src/timeline.rs) demonstrates the composed conversion.

Finite arithmetic is not clamped. Rescaling recovers representable results from
overflowing division or subtraction intermediates. Unrepresentable results return
`TimeErrorKind::Unrepresentable` with the operation category; constructor failures
and decreasing document reports have separate typed categories. These errors are
distinct from a normal unresolved result. Arithmetic uses floating-point rounding.

## Effect timing and relevance

`timing::EffectTiming::try_new(EffectTimingOptions)` validates a Web Animations
Level 1 keyframe effect. Numeric delays and iteration duration use milliseconds;
iteration start is finite and nonnegative. Duration is initially zero and may be
positive infinity. `IterationCount` accepts nonnegative finite counts or explicit
infinity. Either zero factor makes active duration zero, including zero times
infinity. Iteration start offsets progress without extending active duration.
Defaults use normal direction, no fill and linear easing. Invalid numeric options
are rejected by the owning constructor before sampling.

`EffectTimingInput::Associated` supplies the animation's current time as
`Option<AnimationTime>` and coherent `EffectAnimationState` observations. An
unassociated effect has unresolved local time and is idle. Sampling rejects
resolved time for an idle animation and unresolved time for a finished animation. There is at most one
associated animation and no group-effect hierarchy. The owner supplies playback
rate, `EffectPlayState` (idle/running/paused/finished), optional timeline monotonicity and
active/removed/persisted replace state; sampling does not derive host lifecycle.

The active duration is duration times count. End time is
`max(start_delay + active_duration + end_delay, 0)`. The before-active boundary is
`max(min(start_delay, end_time), 0)` and the active-after boundary is
`max(min(start_delay + active_duration, end_time), 0)`. Negative animation playback
rate gives backwards boundary direction: equality is before at the lower bound
and active at the upper bound. Forwards direction includes zero rate, with
equality active at the lower bound and after at the upper bound. Unresolved local
time is idle.

Fill resolves active time without moving those boundaries. Backwards/both fill
before the interval gives `max(local - start_delay, 0)`; the active phase gives
`local - start_delay`. Forwards/both fill after the interval gives
`max(min(local - start_delay, active_duration), 0)`. Other cases are unresolved.
Negative end delay can end the active phase before full iteration progress: with
delay 100 ms, duration 200 ms and end delay -50 ms, forwards fill at local time
250 ms yields active time 150 ms and progress 0.75. The indefinite-fill advice
does not reject an otherwise valid fill option.

Sampling derives overall progress, simple progress, current iteration, directed
progress and eased progress in that order. Exact final iteration endpoints hold
1 rather than wrapping to 0 when count is nonzero. At the full active duration,
the configured count preserves the terminal iteration despite product rounding.
Source-defined infinite
overall progress uses `iteration_start % 1`; infinite current iteration selects
forwards direction in both alternating modes. Easing uses the existing easing
owner and sets its before flag only for before-phase forwards direction or
after-phase reverse direction. The [model example](../src/timing/effect.rs)
demonstrates construction and sampling.

`EffectTimingSample` exposes the supplied typed local time and read-only derived
observations. Its predicates follow the
[selected timing clauses](../../../references/web-animations-1--WD-web-animations-1-20230605--e814a799ef24.md#animation-effect-phases-and-states):

| Predicate | Required observations |
| --- | --- |
| `is_in_play` | Active phase with an associated animation that is not finished. A paused animation can meet this predicate. |
| `is_current` | In play, forward-before, backward-after, or a nonidle animation with an associated nonmonotonic timeline. |
| `is_in_effect` | Resolved active time, including applicable fill. |
| `is_relevant` | Current or in effect, with an associated animation whose replace state is not removed. |

One private phase/progress kernel serves this millisecond model and the
seconds-based normalized CSS `TimingParameters`/track samplers. CSS compares
delay-relative time, including at a start where adding a short duration would
round to the delay. The CSS samplers retain their nonnegative effective
elapsed-time domain, including finite second values
too large to convert into a finite millisecond coordinate. Shared observations
reuse `TimingPhase`, `CurrentIteration`, `PhaseProgress` and `EasedProgress`.
`TimingSample::is_finished()` describes a normalized forwards elapsed-time track;
the general effect predicates use the supplied animation play state instead.

Finite arithmetic limits remain distinct from source-defined infinity.
`EffectTiming` returns a canonical `EffectTimingError` carrying
`EffectTimingErrorKind::Unrepresentable` and its operation; invalid options have
a separate category. The CSS timing observation exposes the diagnostic through
`timing_error()` while actual CSS tracks preserve their existing phase-based
classification and no-value result when progress is unavailable. A later
representable input can recover. In particular, overflow of finite
`active_time / duration` is a typed numeric limit, not infinite overall progress.
Sampling has no frame history: seeks and cadence do not accumulate error, while
new timing options and supplied play observations explicitly affect new results.

## Sampling contracts

`KeyframeTrack::sample_into(elapsed, play_state, &mut output)` replaces an
existing `Vec<SampledPropertyResult>` with one owned result per property, in
first-appearance order. It retains the buffer's capacity and allocates nothing
during primitive sampling when capacity is sufficient, including unsupported
and failed results. Computed component records and symbolic plans allocate their
new immutable result storage; they do not share the primitive zero-allocation
promise. `sample()` returns the same ordered results in a newly allocated vector.
Tracks retain their original `segments()` order independently of their lookup
index. Sampling remains stateless; retained results do not borrow their track.

Each `KeyframeSegment` carries the easing of its starting keyframe. It governs
the interval to the next keyframe for that property, including a caller-supplied
implicit 100% endpoint. Callers omit the timing function on the final keyframe:
it has no outgoing interval. After applying direction, sampling selects each
property's interval and applies its easing once to the local interval fraction.
Reverse playback traverses those same intervals and timing functions backwards.
During the delay, backwards fill holds the first iteration's directed starting
value without applying easing. At the start of the active phase, interval easing
applies normally. CSS endpoint synthesis and CSS keyframe assembly remain caller-owned.

## Native keyframe normalization

`keyframe_input::KeyframesInput<V, E>::normalize()` consumes an absent,
sequential, or property-indexed candidate. `V` is the caller's accepted parsed
value, including any typed source provenance; `E` is its easing validation
failure. The raw inputs expose fields because they can contain invalid data.
Only normalization constructs `NormalizedKeyframes<V>` and its private-field
frames. Results preserve specified offsets separately from computed offsets,
validated easing, optional composite overrides, and accepted property payloads.

Each property entry uses a caller-supplied canonical `PropertyKey` and an
`Option<V>`: `Some` retains the payload unchanged, while `None` discards a
rejected property value without removing its frame or list slot. Duplicate keys
in one typed dictionary are rejected. Property identity is exact and
case-sensitive, including custom properties; this API does not convert IDL
names or choose animatable properties. Entry order within merged frames follows
caller track order. Scalar property-indexed metadata is represented by a
one-element list; empty easing defaults to linear and empty composite to auto.

Offsets are raw `Option<f64>` inputs. The complete explicit sequence is checked
for loosely nondecreasing order before checking finite `[0,1]` range, and both
checks precede easing failures. Equal sequence offsets remain separate frames in
their supplied order. A missing first offset defaults to zero when there are
multiple frames; a missing final offset defaults to one, including a singleton.
Interior missing runs are distributed by frame index between defined anchors.
Specified anchors retain their exact validated values.

For property-indexed inputs, each property track first receives intermediate
computed offsets. Tracks are stably sorted and merged at equal intermediate
offsets. The supplied offset list is assigned positionally to these merged
frames, without repetition; surplus offsets are ignored. Final computed offsets
are calculated anew from specified offsets. For example, tracks of lengths
three and four form five merged frames; without specified offsets the final
computed offsets are `0, .25, .5, .75, 1`, while property entries retain the
placement determined by the intermediate merge.

Short easing and composite lists repeat independently. Every supplied easing
validation result is checked, including surplus entries when no frame exists.
Used entries precede surplus entries for failure reporting. `None` composite is
auto; `effective_composite(effect)` resolves it against the current effect
setting without freezing that setting during normalization. Replacement
publication remains caller-owned: normalization returns the complete result or
a `KeyframeInputError<E>` with its category, source position, and applicable
offset, property, or original easing failure. It publishes no partial frames.
When `E` implements `std::error::Error`, the normalization error also implements
that trait and exposes numeric or easing causes.

These rules implement the pinned [native processing algorithm](../../../references/web-animations-1--WD-web-animations-1-20230605--e814a799ef24.md#processing-a-keyframes-argument)
and [missing-offset calculation](../../../references/web-animations-1--WD-web-animations-1-20230605--e814a799ef24.md#compute-missing-keyframe-offsets).
CSS parsing and serialization, target-context value computation, computed-frame
sampling, and effect replacement are separate caller or consumer boundaries.

Property names, value markers, and diagnostic endpoints use immutable shared
storage. This moves some allocation into construction; see the
[sampling measurements](performance.md) and
[construction measurements](construction-performance.md) for setup costs and throughput.

Single-property tracks validate and search their original segment slice without
allocating grouping metadata. The private two-property `Pair` representation
shares an immutable `Arc<[PropertySegmentGroup; 2]>` across track clones. A third
property promotes the completed prefix into an owned general group vector,
initially reserving four group slots and eight temporary `HashMap` entries.
The collections grow as needed.

Both grouped representations retain cached property keys in first-appearance
order, with segment membership stored as contiguous ranges or scattered indices.
The first interruption of a range promotes it to indices. Construction completes
grouping before validating properties, preserving the first diagnostic for
competing malformed inputs. Track clones and returned samples remain fully owned.

## Text intake

| Types | Borrowed text | Existing shared text |
| --- | --- | --- |
| `PropertyKey`, `DiscreteValue` | `from_text(&str) -> Result<Self, InterpolationError>` | `from_shared(Arc<str>) -> Result<Self, InterpolationError>` |
| `TransformValue`, `CompositeValue` | `unsupported_from_text(&str) -> Self` | `unsupported_from_shared(Arc<str>) -> Self` |

Property and discrete text reject empty or Unicode-whitespace-only contents
with their existing typed errors. Valid text is preserved exactly, including
surrounding whitespace. Blank unsupported markers retain the `"unsupported"`
fallback. Borrowed validation happens before allocation; valid borrowed intake
uses one shared-storage allocation, and valid shared intake uses none. Blank
marker fallback may allocate. All existing constructor signatures remain valid.
Equality and hashing retain their content semantics wherever those traits are
implemented. Owned values and their clones remain usable after caller text is
dropped and across threads. See [key preparation](how-to.md#prepare-property-keys-and-value-text).

## Computed numeric combination

`computed::ComputedValue` holds a complete property's ordered components.
`interpolate`, `add`, and `accumulate` preserve operand order and return complete
results. Different component counts, incompatible types, or differing discrete
components switch the entire property at progress 0.5. Addition/accumulation of
such incompatible records selects the second record. Compatibility is checked
before arithmetic, so a later discrete conflict cannot leave an earlier numeric
component partially interpolated. Callers select property-specific animation
types and expand authored shorthands; this is the by-computed-value operation.

`ComputedNumeric` admits finite numbers and percentages, computed integers,
canonical dimensions, dimension-percentage mixes and contextual math sources.
`DimensionKind` fixes units to px, deg, s, Hz or dppx. Percentages are unrestricted
finite ratios. Compatible mixes combine both terms independently, normalize
zero terms, and retain percentages without consulting layout. Number-percentage
mixes and different dimension kinds are incompatible. `ComputedPosition` accepts
only length/percentage x/y offsets already normalized from the top-left corner;
its axes combine independently. Authored edge keywords and writing-mode mapping
belong to caller lowering.

`ComputedInteger` stores the integral finite binary64 values. Caller lowering
converts exact authored quantities into this declared computed precision.
Interpolation rounds nearest with ties toward positive infinity, so -1.5 becomes
-1. Addition and accumulation use checked finite arithmetic. Values beyond a
property's range remain valid intermediates: callers clamp only after their
complete interpolation/addition/accumulation sequence.

`ComputedRatio` preserves the supplied nonnegative pair, including zero and
positive infinity. Either zero or infinite component makes interpolation
degenerate, including at exact endpoints. `compare` compares quotients without
floating-point cross-product overflow, returning `None` for 0/0 and
infinity/infinity. Component equality and quotient comparison are distinct.
Nondegenerate interpolation is geometric in the quotient, uses logarithms, and
returns result/1; exact endpoints retain their original pair. Ratios are not
additive, so addition/accumulation selects the second ratio. A degenerate ratio
causes whole-property discrete interpolation, while other compatible components
still add normally during addition/accumulation.

`MathContext` identifies a caller's complete immutable resolution context;
`context.source(type)` creates a distinct identity for one computed calculation
there. Clones preserve identity. The caller retains the mapping from each
`MathSource` to its original calculation and context for as long as unresolved
results are needed. These handles do not store authored CSS or evaluate it.
`math_nodes()` exposes a flat postorder plan whose last node is the result.
Every combination node refers only to preceding operand indices and records its
numeric type and operation. Integer interpolation nodes include integer rounding.
The plan represents symbolic calc composition without duplicating a CSS parser
or function inventory. Construction uses checked allocation/index arithmetic,
has no arbitrary resource cap, and returns `Capacity` on reserving failure.

Use `InterpolableValue::computed(record)` in ordinary transition/keyframe
endpoints. A computed sample-local failure produces `SampleClassification::Failed`
and `InterpolationError::Computed`, retaining both endpoints and a semantic
`ComputedError` with its operation and component index. Nonfinite arithmetic,
ratio underflow and capacity refusal are distinct error kinds. Later endpoint
samples can recover. See [the computed-value example](how-to.md#combine-computed-property-values).

## Integration boundary

Use the public module paths for API access. The modules expose typed time,
progress, easing, timing, interpolation, transition, keyframe, sample, and
diagnostic contracts; `transition::TransitionTrack` and `keyframe::KeyframeTrack`
are the entry points for sampling.

Effective elapsed time is the non-negative finite `ElapsedTime` supplied by
runtime/root after clocks and track start times have already been resolved. This
crate does not own clocks or scheduling.

Paused sampling is explicit through `AnimationPlayState::Paused`. Paused
pending, active, and failed samples are stable until external input; running
before and active samples can report `NextFrameHint::MayChange`.

Root-supplied normalized tracks are the input shape for this crate. Root/style
choose transition generation, keyframe lookup, endpoint synthesis, property
support policy, and list pairing before constructing transition or keyframe
tracks here.

`LinearControlPointInput` holds raw input/output coordinates for CSS `linear()`.
`LinearEasing::new` validates their finiteness, input ordering, and normalized
boundary outputs before publishing read-only `LinearControlPoint` values through
`points()`. Rejection retains the input index and component in its typed error.
Accepted coordinates retain their exact values, including signed zero and
duplicates. Linear easing supports duplicate input selection, interpolation,
and extrapolation, including unrestricted finite `EasingInput` values.

`TimingSample` and `SampledPropertyResult` expose read-only sampling observations;
callers cannot independently replace their coupled fields. Standalone value and
diagnostic enums remain available for inspection and comparison. Finite progress,
property keys, and value-family markers express their documented generic
contracts; property support, lifecycle, and clock policy remain consumer-owned.

Unrestricted percentages are finite computed ratios, where `1.0` means 100
percent but values below 0 or above 1 remain valid. Property-specific percentage
range policy is not applied in this crate.

Premultiplied color interpolation is explicit. `InterpolableColor` records the
rectangular interpolation space (`Srgb` or `Oklab`), premultiplied components,
and alpha. Root supplies values in the intended space. This crate interpolates
premultiplied components and alpha directly.

Downstream clamping and color conversion are root/consumer responsibilities.
This crate does not clip values, gamut-map, unpremultiply, resolve missing color
components, convert between spaces, or apply final property range policy.

Permanent unsupported behavior is a stable unsupported outcome.
Recoverable sample-local arithmetic failure is a failed sample that can later recover.
Unsupported same-family interpolation, such as transform or composite
interpolation in this crate, produces `SampleClassification::Unsupported` and
`SampleValue::Unsupported`. Finite arithmetic overflow during an otherwise
supported interpolation produces `SampleClassification::Failed` and
`SampleValue::Error`, and later endpoint samples can recover.

## Capability table

| Capability | Status | Owner / behavior |
| --- | --- | --- |
| Time intake, finite durations, delays, and effective elapsed time | `supported` | `surgeist-animation` validates finite typed time values; runtime/root supplies elapsed time. |
| Signed nullable model time, immutable conversion snapshots and fixed-origin document time | `supported` | `surgeist-animation` owns pure coordinate arithmetic and checked document reports; callers supply clocks, activity and timeline identities. |
| Timing phases, fill modes, direction, iteration counts, play state, and next-frame hints | `supported` | `surgeist-animation` samples validated timing and reports stable or may-change hints. |
| Web Animations effect timing, signed local time, end delay, iteration start and relevance | `supported` | `EffectTiming` samples supplied animation observations through the shared phase/progress kernel without owning animation lifecycle or target queries. |
| Typed native keyframe inputs and missing-offset computation | `supported` | `keyframe_input` normalizes sequences and property-indexed inputs while preserving specified offsets and caller payloads. |
| Cubic-bezier, steps, linear keyword, and CSS `linear()` easing | `supported` | `surgeist-animation` evaluates typed easing, including canonical linear control points. |
| Number, unrestricted percentage, and explicit premultiplied color interpolation | `supported` | `surgeist-animation` interpolates finite values with checked arithmetic and explicit color spaces. |
| Discrete value switching | `supported` | `surgeist-animation` switches discrete values at the interpolation midpoint. |
| Computed numeric records, normalized positions, ratios and symbolic math | `supported` | Typed computed components combine without contextual evaluation or early clamping. |
| Transform and opaque composite-marker interpolation | `typed-unsupported` | Markers produce typed unsupported samples rather than fallback values. |
| CSS parsing, cascade, transition generation, keyframe lookup, endpoint synthesis, property transitionability, allow-discrete policy, and shorthand/list expansion | `deferred/root-owned` | Root/style normalize authored declarations into this crate's typed tracks. |
| Runtime clocks, scheduling, lifecycle, invalidation, pause bookkeeping, and external input changes | `deferred/root-owned` | Runtime/root decide when to sample and pass `ElapsedTime` plus play state. |
| Color-space conversion, gamut mapping, clipping, and final property range clamping | `root-owned` | Root/consumers prepare interpolation-space values and apply downstream output policy. |
| Root facade reexports, root integration tests, workspace wiring, and API artifact refresh | `root-owned` | These surfaces share the repository; inspect root source and current verification evidence for implemented integration. |

## Verification and attribution sources

[AGENTS.md](../AGENTS.md) lists local verification commands;
[How-to guides](how-to.md) describes running them.
[NOTICE.md](../NOTICE.md) records third-party source attribution coverage, while
[LICENSE](../LICENSE) contains the project's license.
