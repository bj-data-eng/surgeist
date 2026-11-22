# Reference

## Package and public interface

[Cargo.toml](../Cargo.toml) declares `surgeist-animation` 0.1.0, Rust edition 2024,
and the library import name `surgeist_animation`. It declares no runtime
dependencies, features, or explicit minimum Rust version. The dev-only
`stats_alloc` dependency instruments allocation measurements.

[src/lib.rs](../src/lib.rs) declares the public modules `computed`, `diagnostics`,
`easing`, `interpolation`, `keyframe`, `keyframe_input`, `playback`, `progress`,
`sample`, `time`, `timeline`, `timing`, `transform`, and `transition`, and the
constant `CRATE_NAME`. Each public type has one path through its owning module,
such as `surgeist_animation::easing::Easing`. The crate forbids unsafe code. Tests live in
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

## Playback ownership and coordinate observation

`playback::AnimationRecords` owns one animation creation-order universe across
the caller's documents. Creating a record sets unresolved start and hold times,
rate one and optional timeline/effect associations. The records boundary exposes
idle association changes and retirement, with no controls that change playback
time. Association changes therefore have no pending readiness or finished work
to process in any state constructible through this API.

Opaque `AnimationId` and `EffectId` handles retain their owner identity. Foreign
handles and retired registrations return separate typed errors; removed slots
are never reused. Creation order remains stable across transfers, timeline
reassignment and other records' retirement. `animations()` enumerates live
records in creation order. There is no process-global static registry: consumers
share the owner for their global animation list.

`register_effect` accepts validated `EffectFacts`, either derived from
`EffectTiming` or constructed with a nonnegative finite/infinite end time.
`create` and `assign_effect` enforce at most one animation per effect. Assigning
an already-owned effect detaches its previous animation, and identical assignment
is a no-op. Invalid requests and exhausted identity/capacity checks preserve the
existing ownership graph. Retirement releases registration ownership; an effect
must be detached before its registration can be retired. This is not `cancel()`
and does not destroy host objects or dispatch lifecycle events.

`snapshot(animation, timeline)` checks supplied timeline identity against the
record's association. Missing required facts, a mismatched timeline and an
unexpected timeline on a detached record are errors. A supplied inactive
timeline is valid and yields unresolved current time. Snapshots own their
captured associations and effect facts, which remain unchanged through later
reassignment or retirement.

`PlaybackCoordinates` is a separate immutable value combining the existing
`AnimationSnapshot` and optional held animation time. Current time uses hold
first, including when both start and hold are resolved. Otherwise the existing
checked timeline-to-animation conversion supplies current time. Missing/inactive
timeline or unresolved start yields unresolved; arithmetic limits remain typed
errors. This observes supplied state without performing seek, clamping or a
playback state transition.

`observe` derives the no-pending-task play-state predicates in source order and
produces a read-only `PlaybackObservation`. Paused takes precedence over
finished; zero rate does not imply paused. The observation builds a coherent
`EffectTimingInput` for an associated effect, or an unassociated input when there
is no effect. Callers cannot independently overwrite its play state and current
time. See the [coordinate example](../src/playback.rs).

Full playback control remains outside this boundary: pending play/pause and
readiness generations, current/start-time setters with their pending/finished
effects, seamless rate changes and reverse; then finished-state updates,
completion identities, finish/cancel and ordered deferred/event requests.
Runtime retains actual scheduling, coherent publication, event dispatch and
document-facing API projection.

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

## CSS transition lifecycle decisions

`transition::TransitionSnapshot<T>` holds immutable records keyed by a caller's
target identity and canonical longhand `PropertyKey`. Its private record
collection contains one `TransitionState::Running` or `Completed` per key;
`new(records)` rejects duplicates across both kinds. Running records are created
only by successful decisions and retain original used timing, effective timing,
signed start/end coordinates, start/end values, reversing-adjusted start value,
and shortening factor. Completed records preserve the end target to suppress
unrelated retriggering, plus completion-event timing metadata.

`snapshot.update(now, changes, policy)` first promotes every run whose end time
has been reached, then evaluates the supplied atomic style-change batch against
that promoted state. An empty batch advances completion alone. The result's
`after_completion()` and `completions()` let Runtime publish all promotions
before a same-time style event or completion dispatch. `state()` and `outcomes()`
carry the final candidate and ordered cancellation, creation, and completed
removal outcomes. This crate neither publishes state nor dispatches events.
Runtime owns revision checks and application of the captured candidates.

Style supplies `TransitionChange` eligibility, coherent before/after computed
values, and an optional last-match `TransitionTiming`. Duration and delay list
matching, shorthand expansion, document presence, and declarative-animation
advancement belong to Style. `eligible` means the element has both applicable
style snapshots; ineligible entries do not initiate or retarget transitions.
Declarative-animation-only changes do not initiate transitions because Style's
before snapshot already advances those animations to the same time.

The supplied `TransitionPropertyPolicy` determines compatibility of the actual
value pair and performs interpolation using the selected property animation
type. A generally continuous property can have a discrete particular pair;
neither discrete nor not-animatable pairs initiate transitions. The decision
engine samples a changed running target internally at the event time, then asks
the same policy about current-to-after compatibility. External stale samples
cannot be substituted. Policy operations must be deterministic and must not
publish state. Existing `InterpolationPair` algorithms may implement supported
property types; unsupported interpolation remains a typed capability failure,
distinct from declaring a property not animatable. This boundary does not add
ratio, position, integer, transform, or other deferred property algorithms.

New timing applies to creation, restart, and reversal. A matching running target
that remains unchanged retains its original timing even if new duration/delay
would prohibit creation. Removing its matching item cancels it. Changed targets
equal to the current value, incompatible pairs, and nonpositive combined duration
cancel the old run. Ordinary restarts begin at the old current value with factor
one. Reversal preserves the old end as its new reversing-adjusted start and uses
the absolute, clamped expression `oldOutput * oldFactor + (1 - oldFactor)` as
the new factor. Duration is shortened; only negative delay is shortened.

`RunningTransition::sample(now, policy)` is stateless. Signed
`OriginRelativeTime` coordinates use milliseconds; duration and delay use their
existing second-based types, with checked conversion. Through exact start time,
sampling returns the raw start value without interpolation. During duration it
applies easing once to `(now-start)/(end-start)` before property interpolation.
Old timing-function output is retained separately from current value for
reversal, including delay samples. At or after end, the record contributes no
value; completion promotion retains its target separately. Zero-duration runs
with positive delay hold their start until the coincident start/end, then complete
without division. Property values and easing overshoot are not clamped here.

Unresolved `now` returns `Ok(None)` from update and an unresolved sample from
sampling. It never means zero and makes no time-dependent decisions. Duplicate
keys or changes, numeric limits, collapsed positive timing, and interpolation
failures return `TransitionDecisionError`; no partial update or promotion is
exposed. Previous snapshots remain unchanged and usable. Document removal is a
separate pure `remove_targets` operation that cancels runs and removes completed
records even without a resolved clock. Runtime supplies removal/update order,
retains event-dispatch bookkeeping, and applies the outcomes before dispatch.

The [pinned transition algorithm](../../../references/css-transitions-1--WD-css-transitions-1-20260108--969d35d13ee4.md#starting),
[current-value rules](../../../references/css-transitions-1--WD-css-transitions-1-20260108--969d35d13ee4.md#application),
and [completion order](../../../references/css-transitions-1--WD-css-transitions-1-20260108--969d35d13ee4.md#complete)
govern these decisions. CSS cascade insertion and Animation suppression remain
Style-owned; event construction, dispatch, and host scheduling remain Runtime-owned.

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
`transition::TransitionSnapshot` owns immutable CSS transition decisions, and
its `RunningTransition` records sample absolute document-origin-relative time
with an explicit property interpolation policy.

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
Unsupported same-family interpolation, such as diagnostic transform markers or
composite interpolation, produces `SampleClassification::Unsupported` and
`SampleValue::Unsupported`. Finite arithmetic overflow during an otherwise
supported interpolation produces `SampleClassification::Failed` and
`SampleValue::Error`, and later endpoint samples can recover.

## Resolved 2D transforms

[`transform`](../src/transform.rs) owns `ResolvedTransformList`, `TransformFunction`,
`TransformMatrix`, `CssPixels`, `ScaleFactor`, `TransformAngle` and their numeric
diagnostics. Scalars and matrix coefficients are finite and immutable. Finite
negative lengths/scales, zero scales and singular matrices are valid. Matrices
use CSS column-major `[a,b,c,d,e,f]` order. Lengths are resolved CSS pixels;
angles have explicit radians, degrees or turns constructors and retain turns.
Function variants preserve CSS names and optional argument arity.

`TransformValue::resolved(list)` enters the normal interpolation, transition and
keyframe sampling paths. `resolved_list()` borrows the immutable payload.
Unsupported marker constructors never parse their text: even `resolved-2d`
remains an unsupported marker. `kind()` is descriptive text; it does not certify
a concrete payload. Both endpoints must carry resolved lists for interpolation.

An empty list represents `none`. Sampling pads the shorter list with matching
identity functions. Functions with matching name/arity interpolate numerically;
translate/scale derivatives normalize to their common 2D primitives. Different
skew axes have no shared primitive. At the first unmatched pair, each remaining
suffix is post-multiplied into a matrix and interpolated as one matrix. Matching
prefixes retain their function semantics, including full rotation turns.
`matrix()` pairs use decomposition even when their names match.

Matrix sampling follows the pinned [Transforms 1 algorithm](../../../references/css-transforms-1--CR-css-transforms-1-20190214--4672e86844bf.md#interpolation-of-2d-matrices):
decompose translation, signed scales, angle and residual; normalize flipped axes;
adjust matrix rotations to the shorter arc; interpolate components and recompose.
[#552](https://github.com/bj-data-eng/surgeist/issues/552) selects direct placement
of extracted affine translation in recomposition. The pinned residual-weighted
translation does not reconstruct matrices with shear. The selected midpoint of
`matrix(1,0,1,1,10,20)` and `matrix(1,0,1,1,30,40)` is
`matrix(1,0,1,1,20,30)`, independently witnessed in Firefox 152 and corroborated
by [pinned WebKit affine recomposition](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/platform/graphics/transforms/TransformationMatrix.cpp#L1878-L1893).
This bounded selection does not adopt newer 3D algorithms.

Exact singularity of finite matrix endpoints selects whole-value discrete
fallback: the original from value below progress `0.5`, the to value otherwise.
The determinant sign compares exact binary significand products, without an
epsilon or overflowing/underflowing floating determinant products. A singular
intermediate sample remains a value for caller-owned visibility. Column
directions are normalized at unit magnitude before restoring scale, so
rounded subnormal lengths cannot distort the other axis. An established singular
matrix pair selects fallback before arithmetic in a later unmatched suffix.
Scalar, decomposition, composition and recomposition overflow instead produce typed
sample-local arithmetic failure with a function index and transform component.
Exact zero/one progress returns the original concrete endpoint. Finite eased
progress outside the unit interval is preserved; output is never clipped.

`ResolvedTransformList::added_to` concatenates lists in operand order, retaining
all function identities. `to_matrix()` performs checked CSS post-multiplication.
Parameter accumulation, 3D values, authored calculations, percentage/reference-box
resolution, transform origins and visibility decisions are outside this
increment. Callers resolve context before construction and refresh the immutable
endpoints when it changes; no percentage basis is guessed.

## Capability table

| Capability | Status | Owner / behavior |
| --- | --- | --- |
| Time intake, finite durations, delays, and effective elapsed time | `supported` | `surgeist-animation` validates finite typed time values; runtime/root supplies elapsed time. |
| Signed nullable model time, immutable conversion snapshots and fixed-origin document time | `supported` | `surgeist-animation` owns pure coordinate arithmetic and checked document reports; callers supply clocks, activity and timeline identities. |
| Timing phases, fill modes, direction, iteration counts, play state, and next-frame hints | `supported` | `surgeist-animation` samples validated timing and reports stable or may-change hints. |
| Web Animations effect timing, signed local time, end delay, iteration start and relevance | `supported` | `EffectTiming` samples supplied animation observations through the shared phase/progress kernel without owning animation lifecycle or target queries. |
| CSS transition creation, cancellation, reversal, and completion decisions | `supported` | `transition::TransitionSnapshot` consumes coherent caller facts and returns immutable candidates and lifecycle outcomes. |
| Typed native keyframe inputs and missing-offset computation | `supported` | `keyframe_input` normalizes sequences and property-indexed inputs while preserving specified offsets and caller payloads. |
| Cubic-bezier, steps, linear keyword, and CSS `linear()` easing | `supported` | `surgeist-animation` evaluates typed easing, including canonical linear control points. |
| Number, unrestricted percentage, and explicit premultiplied color interpolation | `supported` | `surgeist-animation` interpolates finite values with checked arithmetic and explicit color spaces. |
| Discrete value switching | `supported` | `surgeist-animation` switches discrete values at the interpolation midpoint. |
| Computed numeric records, normalized positions, ratios and symbolic math | `supported` | Typed computed components combine without contextual evaluation or early clamping. |
| Resolved 2D transform interpolation and list addition | `supported` | Typed native lists, identity padding, common primitives, matrix suffixes, whole-value singular fallback and checked arithmetic. |
| 3D transforms, transform accumulation, diagnostic transform markers and composite interpolation | `typed-unsupported` | Markers retain explicit unsupported outcomes; no authored or 3D lowering is implied. |
| CSS parsing, cascade, transition generation, keyframe lookup, endpoint synthesis, property transitionability, allow-discrete policy, and shorthand/list expansion | `deferred/root-owned` | Root/style normalize authored declarations into this crate's typed tracks. |
| Runtime clocks, scheduling, lifecycle, invalidation, pause bookkeeping, and external input changes | `deferred/root-owned` | Runtime/root decide when to sample and pass `ElapsedTime` plus play state. |
| Color-space conversion, gamut mapping, clipping, and final property range clamping | `root-owned` | Root/consumers prepare interpolation-space values and apply downstream output policy. |
| Root facade reexports, root integration tests, workspace wiring, and API artifact refresh | `root-owned` | These surfaces share the repository; inspect root source and current verification evidence for implemented integration. |

## Verification and attribution sources

[AGENTS.md](../AGENTS.md) lists local verification commands;
[How-to guides](how-to.md) describes running them.
[NOTICE.md](../NOTICE.md) records third-party source attribution coverage, while
[LICENSE](../LICENSE) contains the project's license.
