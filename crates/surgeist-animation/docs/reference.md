# Reference

## Package and public interface

[Cargo.toml](../Cargo.toml) declares `surgeist-animation` 0.1.0, Rust edition 2024,
and the library import name `surgeist_animation`. The runtime dependency
`surgeist-color` supplies checked numerical color sampling. The dev-only
`stats_alloc` dependency instruments allocation measurements.

[src/lib.rs](../src/lib.rs) declares the public modules `computed`, `diagnostics`,
`composition`, `css_animation`, `easing`, `interpolation`, `keyframe`,
`keyframe_input`, `palette`, `playback`, `progress`, `sample`, `shadow`, `shape`,
`time`, `timeline`,
`timing`, `transform`, and `transition`, and the
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

## Playback ownership and native controls

`playback::AnimationRecords` owns one animation creation-order universe across
its caller's documents. Creation sets unresolved start and hold times, rate one,
a resolved ready completion and a distinct pending finished completion. Opaque
animation/effect handles reject foreign and retired registrations, and removed
slots are never reused. Creation order stays fixed across transfers and timeline
changes. There is no process-global static registry.

`PlaybackContext` captures a single supplied snapshot per timeline identity.
Duplicate identities are rejected. Commands require facts for each affected
association, including an effect's displaced owner. Missing required facts are
errors; an explicitly inactive timeline is valid. An identical association is a
no-op before fetching new facts. All fallible work happens on affected candidate
records before the owner graph and returned requests commit together.

`register_effect` accepts validated `EffectFacts` derived from `EffectTiming` or
constructed with a nonnegative finite/infinite end time. `create` and
`apply(AssignEffect)` enforce at most one owner per effect. Transfer detaches the
previous owner and updates its pending readiness and finished state.
`update_effect` supplies changed timing/setup facts, advances their checked
revision, reschedules affected readiness and updates finished state. Its revision
changes even when end time is equal, since effect setup may have changed.

`create(timeline, effect, context)` returns `CreatedAnimation`: its animation
identity plus the initial completion registrations and any displaced-owner
requests. `apply(animation, command, context)` is the canonical mutating front
door and returns `Vec<PlaybackRequest>` in source order:

| Command | Behavior |
| --- | --- |
| `Seek`, `SetStartTime` | Apply the complete seek/start-time procedures, pending rate application, synchronous pause completion and finished-state update. Unresolved seek is rejected with native TypeError when current time is resolved. |
| `Play`, `Pause` | Select source rewind/hold/start behavior and schedule deferred readiness. Pending task replacement retains the pending ready completion identity. |
| `SetPlaybackRate` | Change committed rate synchronously, preserving current time on monotonic timelines and applying the source finite-timeline sign-flip rule. |
| `UpdatePlaybackRate`, `Reverse` | Stage seamless changes, synchronize through pending readiness, or use the finished/idle branches. Reverse requires an active timeline and failures preserve the original staged rate. |
| `Tick` | Update finished state from the supplied frame, using unconstrained time so nonmonotonic timelines can leave finished. |
| `Finish` | Seek to the direction's limit, perform applicable ready settlement and synchronously request finished settlement/event queueing. Zero effective rate and positive infinite end are InvalidStateError cases. |
| `Cancel` | Clear current time, retain effect association, reset pending tasks and return ready/finished abort and replacement outcomes plus a cancel event. Initial idle cancellation produces no notification. |
| `AssignTimeline`, `AssignEffect` | Apply association changes, reschedule affected readiness and update finished state. |

The source current-time calculation gives hold time precedence over start time,
including when both are resolved. Start assignment captures previous current
time after its inactive-timeline hold adjustment but before applying staged rate.
Normal playback clamps at the direction's boundary; an explicit seek preserves
overshoot as hold time. Paused precedes finished, and zero rate alone does not
imply paused. Registry snapshots derive their full observations from pending
state and effective rate, with no independently editable observation flags.

### Deferred requests and completion identity

`ScheduleReadiness` always means asynchronous execution, even if the effect is
immediately ready. Runtime owns the actual processing point. Capture the
`ReadinessToken` and return it to `complete_ready` with a `ReadinessInput` carrying
the active timeline timestamp when setup/suspension became ready. This timestamp
can precede the dispatch context timestamp. Readiness rejects stale generations,
foreign owners, mismatched identities, unfinished setup and contradictory
monotonic time/capability reports without consuming the token. Effect/timeline
changes invalidate old readiness generations while preserving the current
pending ready completion. Completing readiness resolves ready before invoking
its finished-state update.

The selected Level 1 play steps 9–10 cancel pending work before their redundant
play return. On a nonmonotonic timeline with resolved start and unresolved
hold/seek, repeating play can therefore leave the reused ready completion
pending with no pending task. The ordinary public-call witness is retained in
[the controls tests](../tests/playback_controls.rs). The pinned
[WebKit witness](../../../references/style-animation-host-determinations-webkit-native-animation-host--73aa6c89e2cb.md)
adds a Level 2 pending-auto-aligned-start readiness condition with an explicit
FIXME. That separate producer/source behavior is not imported here.

Ready starts resolved and is replaced when new work becomes pending from a
nonpending state. Pending play/pause replacement reuses that identity.
Cancellation with a pending task applies its staged rate, aborts old ready and
creates a resolved replacement; without a pending task ready/rate stay unchanged.
Finished starts pending. Leaving finished after resolution creates a fresh
pending identity. `CompletionState::Aborted` is the native AbortError outcome;
resolved completions are never rewritten as aborted.

`ScheduleFinishNotification` has a distinct token with different semantics.
`process_finish` rechecks live state and uses the **current** finished completion.
Timing edits or cancellation do not automatically invalidate an already queued
notification. Entering and then leaving finished before its callback produces
no finish settlement/event. Synchronous `Finish` cancels the queued token and
notifies immediately; a later callback with that token is stale. One live
notification per animation is deduplicated. These rules retain the selected WA1
[finished-state procedure](../../../references/web-animations-1--WD-web-animations-1-20230605--e814a799ef24.md#updating-the-finished-state).

`QueueEvent` carries an immutable animation-coordinate current time, timeline
coordinate, captured conversion snapshot, creation order and a separate scheduled
origin-relative sorting key. Finish converts associated-effect end through the
#588 animation-to-origin wrapper; cancel converts timeline time directly. An
inactive or missing timeline gives unresolved timeline time and sorting key.
A resolved zero remains distinct. Runtime owns document routing, native event metadata,
stable sorting, dispatch and completion adapters under #594/#595/#596. Model
commands and timing edits do not manufacture a new document frame.

### Snapshots, numeric limits and retirement

`snapshot(animation, timeline)` retains its existing identity-checking input
shape. It returns read-only coordinates, effect/setup revision, pending tokens,
completion snapshots and a coherent full `observation()`. Archived values remain
unchanged after later model transitions.

`PlaybackCoordinates` remains the independent immutable start/rate/timeline plus
optional hold value. It reuses checked #588 coordinate math. Its `observe` helper
represents a supplied state **without pending work**; registered animations use
`AnimationRecordSnapshot::observation()` so pending pause/play and staged rates
participate in play-state derivation. Both observations produce guarded #589
`EffectTimingInput` values.

Failures preserve all affected records, associations, identities and outstanding
tokens. Arithmetic retains #588's typed errors, with a separate unrepresentable
finite sign-flip start-time category. Identity/revision/generation counters use
checked increments. Retirement releases a neutral idle registration without
pending readiness or finish notification; active records must first complete
cancellation/deferred processing. Retirement is not cancel and does not destroy
host objects. An associated effect must be detached before its registration can
be retired.

See the [module example](../src/playback.rs) for context-aware creation and play.

## Effect stacks and replacement

`composition::EffectStackSnapshot::new` validates one immutable set of supplied
animation/effect observations against an `AnimationRecords` owner. It checks
identity, association, effect setup identity/revision, registered effect-end
facts, timeline agreement and effect timing's rules. Adapters supply the truth of
target identities (including pseudo-elements), CSS ownership, document membership,
and class-specific ordering. Local time, play state and playback rate come from
the checked `AnimationRecordSnapshot::observation()`, including pending controls;
stack input accepts only caller-owned replacement state. A cancellation cannot
be bypassed with an independently supplied Finished state or old local time.

`EffectStackEffect::new` binds computed frames to an opaque `EffectSnapshot`
captured when computing them. Reassociation and setup edits invalidate that
bundle even if the new associated effect has the same duration. The computing
adapter still owns freshness for target-context computed values. Old validated
stack snapshots remain immutable through later control or setup changes.

`AnimationClass` identifies the creating interface, independently of CSS owning
elements. `AnimationOrder` first compares applicable inter-class ranks, then
common-class ranks, then checked creation positions. Missing or equal ranks fall
through. CSS adapters supply only ranks admitted by their selected clauses;
orphan CSS animations do not retain owning-element precedence. Undefined orphan
idle order is accepted outside the ordered stack. Every defined pair is checked
after bounded insertion; cyclic or inconsistent facts return a typed error.
No arbitrary caller comparator is passed to library sorting.

`sample` folds represented, animatable, in-effect, nonremoved property effects
from lower to higher order. Each `ComputedKeyframes::sample_timing` receives the
current underlying value and performs endpoint composition before interpolation.
There is no second composition of its returned result. A missing property never
claims contribution, even though the isolated keyframe sampler can return
underlying for an empty property track. The result retains the top contributing
animation and its class for caller-owned cascade insertion. No contribution,
inapplicable target, and typed sampling failure remain distinct.

`EffectStackEffect::not_animatable` excludes represented properties from target
values while retaining their effect timing. Unlisted represented properties are
animatable by default. `AnimationTypeOperations` implements selected discrete,
by-computed-value and repeatable-list operations; property-specific algorithms
can instead implement `PropertyValueOperations`. Guarded Transform payloads use
the transform owner's checked concatenation and accumulation; guarded Shadow
payloads use their encoded box/text kind and the shadow owner's list algorithms.
Guarded Filter payloads delegate ordered concatenation and effect-B accumulation
to `FilterList`, including through default effect-stack endpoint composition.
Shadow and Filter lists never enter generic repeatable-list expansion. Owning
Transform, Shadow and Filter failures retain their typed diagnostics. Unsupported
operations do not
silently become addition. Property-definition and applicable Appendix A
selection remain with the caller, without another CSS property registry.
Guarded Palette payloads delegate nonadditive addition to the palette owner;
unspecified accumulation uses that same effect-operand selection. The default
operations therefore support symbolic palettes in ordinary effect-stack folds.

`replacements` selects Active, non-markup-prescribed, finished animations with a
monotonic timeline and associated in-effect targeted effect in the supplied
document. Every targeted property must be covered on the exact same target by
higher replaceable effects; several effects may jointly cover the set. Persisted
effects can cover lower effects but are not removal candidates. Removed effects
do not cover or contribute. Empty targeted-property coverage is universally
satisfied. All decisions come from the original snapshot in ascending composite
order and retain current and timeline times for Runtime publication and events.
`AnimationReplaceState::persist` unconditionally sets Persisted, including after
removal. Registration retirement remains a separate operation.

`computed::RepeatableList` owns nonempty generic lists of computed items, distinct
from property-specific shadow/transform lists. Combination repeats to the
positive least-common-multiple length with checked arithmetic and reservation.
The complete item structure is checked before arithmetic; any discrete component
or incompatible pair selects an original whole endpoint. This includes equal
discrete components, unlike ordinary by-computed-value combination. Arithmetic,
expansion and capacity failures remain typed errors without partial output.
Final property clamping stays caller-owned.

### Background and mask layer positions and sizes

Both position properties use the existing `ComputedComponent::Position` with
`ComputedPosition` x/y offsets normalized from the top-left corner. Both size
properties use `ComputedComponent::Size(LayerSize)`. Wrap one component in each
`ComputedValue` layer, construct the existing `RepeatableList`, then use
`InterpolableValue::repeatable_list` through ordinary pair/track sampling or
`AnimationTypeOperations::RepeatableList`. No second list algorithm or property
registry is involved.

`LayerSize::Explicit` requires both horizontal and vertical `LayerSizeAxis`
values. Construct each as explicit Auto or a checked numeric Length, Percentage
or LengthPercentage. Number, Integer and other dimension roots are rejected.
Contain and Cover are mutually exclusive whole-size branches. Authored omission,
natural dimensions, aspect ratios, box selection and used size remain with the
caller and geometry consumer; there is no default or zero-size substitute here.

Matching Auto axes remain Auto while the other numeric axes combine. Matching
Contain/Contain and Cover/Cover are constant. These are resolved size states,
not ordinary Discrete components: this preserves numeric interpolation beside
matching keywords as exercised by
[WPT's background-size keyword cases](https://github.com/web-platform-tests/wpt/blob/master/css/css-backgrounds/animations/background-size-interpolation.html).
An Auto/numeric, Contain/Cover or whole-keyword/explicit-size conflict anywhere
in the repeated pairing selects the complete original list before arithmetic.
Interpolation switches at 0.5; addition and accumulation select the original
effect operand. Compatible numeric axes add component-wise, and accumulation
uses the same addition. Layers repeat to the checked LCM rather than concatenate.
Compatible exact-endpoint samples also retain the repeated cardinality.

Background clip/origin continue to use ordinary Discrete components in the
generic RepeatableList. Even equal discrete tags make the whole property switch
between its original endpoint lists. Size constants do not change that rule.
Mask clip/origin are table-discrete properties, with their complete prepared
layer list supplied as one discrete value; they are not these repeatable sizes.

The caller supplies each property's percentage basis: position percentages use
positioning-area size minus image size, whereas size percentages use the
positioning-area dimensions. The selected [Backgrounds position](../../../references/css-backgrounds-3--CRD-css-backgrounds-3-20240311--8a291792b2fd.md#background-position)
and [size](../../../references/css-backgrounds-3--CRD-css-backgrounds-3-20240311--8a291792b2fd.md#background-size)
clauses own those definitions. [Mask position](../../../references/css-masking-1--CRD-css-masking-1-20210805--79ff953b4747.md#the-mask-position)
and [mask size](../../../references/css-masking-1--CRD-css-masking-1-20210805--79ff953b4747.md#the-mask-size)
delegate their value definitions to Backgrounds, so they share these native
types while retaining their caller-supplied mask context.

Percentages, signed intermediate values and symbolic source/context identity
remain unresolved. Negative extrapolated sizes remain reusable until final
caller-owned range clamping after interpolation/addition/accumulation, as required
by [Values range checking](../../../references/css-values-4--WD-css-values-4-20240312--7b6b68c34d00.md#combining-range).
List expansion, allocation and arithmetic errors use existing computed
operation/index diagnostics without partial results. A later finite sample may
recover. See the [layer sampling procedure](how-to.md#sample-repeatable-image-layers).

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
and failed results. Computed component records, shape payloads and symbolic plans
allocate new immutable result storage; they do not share the primitive zero-allocation
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

## Computed keyframe effects

`keyframe::ComputedKeyframes` samples caller-computed values separately from the
contiguous segment API. `ComputedKeyframeInput` supplies optional specified
offset metadata, a required `UnitRatio` computed offset, validated easing,
optional composite override, and longhand/value entries. Construction checks
explicit/computed offset equality, nondecreasing computed order, and unique
property keys per frame. It preserves
equal-offset order without sorting or merging. Read-only `ComputedKeyframe`
records retain specified-offset provenance independently from computed offsets.
Authored offset validation and missing-offset distribution remain in
`keyframe_input`; a supplied explicit offset must equal its computed offset here.
`from_normalized` copies that metadata when the caller's normalized payloads are
already target-context `InterpolableValue`s; it does not resolve authored values.

`KeyframeEndpointSource::Native` inserts missing property endpoints at zero and
one during each sample, using the supplied current underlying value, replace
composition, and linear easing. `CssAssembled` instead requires supplied zero
and one endpoints for each represented property and retains their computed
values, easing, composition, and stable order. Style owns CSS grouping,
generated endpoint assembly, and live target-context computation. Rebuild
computed inputs when their influencing context changes; the sampler does not
cache or invent target geometry or value-resolution context.

`sample(property, input, progress, operations)` accepts optional finite
`InterpolationProgress`, including values outside `[0,1]`. Unresolved progress,
not-animatable properties, and absent or uncalculable targets return `None`.
After filtering frames for the requested longhand, an empty set returns the
underlying value. Missing native endpoints are then inserted. Below zero with
multiple zero-offset frames, the first frame is selected alone; at or above one
with multiple one-offset frames, the last frame is selected alone. Otherwise,
the last offset not greater than progress and less than one (or the last zero
frame) starts the interval, followed by its next frame. Interior exact overlaps
therefore select the last frame at that offset.

Each selected endpoint resolves its override against the effect composite and
composes against underlying before interpolation. A single endpoint returns
directly. For two endpoints, the sampler evaluates the start frame's easing at
`(progress-startOffset)/(endOffset-startOffset)` and then invokes the property's
selected interpolation algorithm. Input and eased output are not clamped;
terminal-frame easing is unused. The selected current Web Animations clause
evaluates interval easing without forwarding an effect timing before flag.

`sample_timing` provides two explicit adapters. Native collections use
`EffectTimingSample::eased_progress`: whole-effect easing already ran and its
before flag was already consumed. It is not forwarded to interval easing, as
the selected current §5.3.4 interval-easing step does not supply that mapping.
CSS collections use directed progress, apply interval easing once, and preserve
the recognized CSS interval step-side flag. During CSS delay fill they bypass
interval easing and hold the directed endpoint. This retains CSS's delay rule
even for easing functions with nonzero initial output. The raw `sample` method
is phase-neutral; callers adapting CSS timing use `sample_timing`.

`composition::PropertyValueOperations` supplies the selected property's pure
interpolation, addition, and accumulation algorithms with its original typed
failure. `compose_value` handles replace directly and passes underlying first to
addition or accumulation. It never substitutes addition for accumulation.
Existing computed operations and resolved 2D transform list addition can provide
their supported mathematics. Discrete fallback and unsupported capabilities
remain the selected property's responsibility; property names or value families
alone are not a replacement for property animation policy.

Sampling is immutable and returns the complete value or `KeyframeSampleError<E>`
with property, operation stage, optional original frame index, and original
easing, progress, or property-operation failure. No partial value is published.
When `E` implements `std::error::Error`, that cause is available through the
standard error chain. The returned value already includes endpoint composition
against its supplied underlying. An effect stack must pass that result directly
as the next underlying value rather than composing it again. Stack ordering,
replacement decisions, cascade insertion, scheduling, and publication remain
separate consumers.

The adopted [current effect-value algorithm](../../../references/style-source-determinations-additional-web-animations-1--editor-capture-20261010--92e1d5e27678.md#the-effect-value-of-a-keyframe-animation-effect)
governs interval selection, endpoint composition, and step 8's interval easing.
Its underlying/replace/linear fallback supersedes the older neutral/add wording.
[CSS assembly](../../../references/style-source-determinations-additional-css-animations-2--editor-capture-20261010--d03de1340087.md#keyframe-processing)
supplies its endpoint override; retained [CSS timing rules](../../../references/css-animations-1--WD-css-animations-1-20230302--f725ad790abb.md#timing-functions)
govern CSS delay and direction behavior.

## CSS animation reconciliation

`css_animation::CssAnimationList` consumes one target's coherent resolved list
and applicability at a native style update. Style supplies list repetition and
truncation, exact name decoding, winning-rule availability, computed rule data,
and used `KeyframeTiming`/`AnimationPlayState`. `CssAnimationSlot::None` is the
keyword; `Unmatched(name)` explicitly means no valid rule; `Matched` contains a
`ResolvedCssAnimation<R>` whose caller-owned payload may represent an empty
valid rule. Every occurrence of a name must agree about match availability.
Rule provenance/revision and computed values remain in `R`, which need not
implement equality or cloning for reconciliation.

`CssAnimationName` is an exact case-sensitive resolved name value. Identifier and
string syntax yielding the same value match. Empty and whitespace string names
remain valid, and a string named `none` remains distinct from the keyword slot.
This name differs from the segment API's `KeyframeAnimationId` and from opaque
`playback::AnimationId`, which identifies one applied playback instance.

`CssAnimationInstance` captures an opaque instance ID, its prior matched
animation, and `PlaybackCoordinates` from the current update. Construction
samples the existing timing and rejects arithmetic failure. Capture all old
instances coherently for the same target in prior animation-name order,
including finished instances that remain applied. Unresolved time remains
unresolved, and held time retains the playback owner's precedence over timeline
arithmetic. Rule resolution and ID liveness remain their respective owners'
facts; Animation rejects duplicate instance IDs in the supplied old list.

`reconcile(previous)` scans new matched slots last to first. Each uses the last
remaining old occurrence of its exact name, consumes it once, and retains its
ID and playback coordinates. A changed rule identity, edited contents, or a
replacement winning rule does not restart that instance. Matched slots with no
old occurrence create start plans at local time zero; missing rules and the
keyword create no animation. Start plans require Runtime to allocate fresh IDs
when published.
Old unmatched instances produce stops. Losing the last matching rule removes
the old instance; after publishing that stop, later rule availability starts
anew. A valid empty rule still creates an instance, including its timing data
for lifecycle consumers.

The caller selects `CssAnimationEligibility`: eligible, element `display:none`,
ancestor `display:none`, or the chosen policy to ignore noninteractive media.
Suppression stops every old instance and creates no starts. Runtime owns ancestor
traversal and supplies the resulting fact; restoration against the published
empty instance list starts eligible animations anew.

The complete immutable `CssAnimationPlan` borrows its inputs. Updates are in
new-slot order, preserving positions skipped by keyword/unmatched slots; stops
are in old-instance order. These orders do not prescribe Runtime event order.
Each update exposes the new rule/timing, optional old instance/index, before and
after `EffectTimingSample`s, and an optional CSS play-state request. Retained
samples evaluate old and new timing at exactly the same supplied local time,
so duration, delay, direction, iteration and fill edits take effect as though
present since start. Timing may jump to after or return to before without
replacing identity. Finished instances remain matchable and their contribution
follows the current fill mode.

Timing samples precede any requested playback operation. Starts request their
initial CSS play-state; retains request it only when the CSS value changed.
Unchanged values leave the existing playback state alone. The playback owner
applies pause/resume requests with timing continuity, then Runtime publishes and
resamples coherently. Reconciliation does not implement a second playback state
machine, mutate animation records, resolve clocks, schedule work, or dispatch
events. Before/after samples, old instance identity, rule updates, stop reasons
and final captured stop samples supply the native publication/event consumer.

`KeyframeTiming` seconds are checked when converted to `EffectTiming`
milliseconds; a finite value is never silently turned into infinity. Both
samples use the existing timing kernel with linear whole-effect easing, since
CSS interval easing belongs to the supplied computed keyframes. Numeric and
sampling failures return `CssAnimationError` with the original typed cause and
applicable old/new index; no partial plan or input mutation escapes.

These rules follow pinned [CSS Animations 1 §2](../../../references/css-animations-1--WD-css-animations-1-20230302--f725ad790abb.md#animations).
Style's named-rule assembly and Runtime's playback publication, scheduling and
lifecycle event dispatch remain separate owners.

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
Interpolation rounds the exact affine result to the nearest integer with ties
toward positive infinity, so -1.5 becomes -1, then represents it at computed
precision. Addition and accumulation use checked finite arithmetic. Values beyond a
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

## Computed Multi-column properties

Style supplies the effective longhands from the pinned
[Multi-column property definitions](../../../references/css-multicol-1--CR-css-multicol-1-20240516--0768f4526ac5.md)
and the [Sizing 4 column-width forms](../../../references/css-sizing-4--WD-css-sizing-4-20260904--faaa86589559.md#column-sizing).
Use the normal transition or keyframe sampling interfaces with these operands.

| Longhand | Operand and sampling behavior |
| --- | --- |
| `column-count` | `ComputedInteger` for counts and `DiscreteValue` for `auto`. Counts round to the nearest integer, with a tie toward positive infinity. A count and `auto` switch at the eased midpoint. |
| `column-width` | `ComputedComponent::BoxSize` wraps a checked `ComputedSize`. Length-percentages and matching functional `fit-content()` arguments combine numerically. Incompatible whole sizes switch at the eased midpoint. |
| `column-gap` | Shared Alignment length-percentage and `normal` sampling. |
| `column-rule-width` | A supplied computed pixel length. Samples may be fractional and retain signed overshoot. |
| `column-rule-color` | A resolved color endpoint and the caller's [selected interpolation method](#standard-space-color-sampling). |
| `column-rule-style`, `column-span`, `column-fill` | Complete `DiscreteValue` endpoints switch at the eased midpoint. |

`CalcSize` separates the basis from a bounded symbolic calculation. It
substitutes nested calculations, reduces a pure-length basis to `any`, and
normalizes a basis containing percentages to `100%`. In that last case,
only percentages originating in the basis become ratios of the bound `size`.
Matching canonical bases combine. An `any` basis adopts the other basis.
An explicit `calc-size()` can also interpolate with a length-percentage or a
compatible intrinsic keyword. The plan preserves the scopes of both `size`
and percentage bindings for downstream evaluation. Preparation limits the
stored plan and expanded calculation to 4,096 nodes. Over-budget pairs switch
as whole values. A failed endpoint preparation can be preserved with
`ComputedSize::unavailable_calc_size`.
See the pinned [calculation preparation and interpolation rules](../../../references/css-values-5-attribute-substitution-definitions--WD-css-values-5-20241111--af984ef26b39.md#interp-calc-size).

Style supplies property animation policy, computed lengths and percentage
contexts, and expands the `columns` and `column-rule` shorthands. Consumers
apply the final property range after composition. Layout owns used column
geometry and Paint owns rule drawing.
The [Multi-column tests](../tests/multicolumn_properties.rs) exercise these
operands through Animation's public interfaces.

## Computed font properties

Callers select the pinned [Fonts 4 property table](../../../references/css-fonts-4--WD-css-fonts-4-20260907--03626a0c8565.md)
animation type and supply effective computed operands. Existing computed numeric
records sample `font-weight`, `font-width`, `font-style`, `font-size` and
`font-size-adjust`. Style supplies normal as comparable oblique zero degrees for
style interpolation, keeps italic as a distinct branch, resolves font-size to
computed lengths, and supplies the `none` branch of font-size-adjust separately
from numbers. Weight samples retain fractions. Font matching and final property
range handling remain outside Animation.

`font-family`, the synthesis longhands/shorthand, kerning, variant
longhands/shorthand, feature settings, language override, optical sizing and
variant emoji select the whole discrete endpoint at the midpoint. For variation
settings, Style supplies sorted, deduplicated effective case-sensitive axis maps:
corresponding numbers interpolate only when their tag sets agree; incompatible
maps and normal versus a map select the whole endpoint discretely. The font
shorthand supplies its effective longhands, and the legacy font-stretch alias
supplies font-width operands. These paths use ordinary interpolation pairs,
keyframe tracks and transition tracks, as demonstrated by the
[font property tests](../tests/font_properties.rs) and
[computed property tests](../tests/computed_layout_properties.rs).

### Symbolic font palettes

`palette::PaletteSource` identifies one caller-owned computed palette operand:
a keyword, named palette or existing computed palette mix. Clones preserve its
identity. Style retains the source-to-value mapping, supplies one identity for
known-equivalent operands and keeps that mapping while plans are in use. A new
source is a distinct operand; Animation does not infer equivalence from font
metadata, rule matching or resolved colors.

`PaletteValue::source` enters that operand into the sampler;
`InterpolableValue::palette` integrates it with pairs, keyframes and transitions.
Strictly interior progress produces an Oklab palette mix. At or below zero it
returns the exact first operand; at or above one the exact second. Equal supplied
operands remain unchanged, including equal already sampled plans. Palette
addition selects the effect palette rather than adding colors. Palette progress
selection is specific to this value family; numeric font interpolation retains
finite extrapolation. The default by-computed-value operations delegate palette
addition to `PaletteValue::add`; unspecified accumulation uses the same
nonadditive selection of the effect palette. There is no separate palette
accumulation algorithm or per-entry color addition.

`PaletteValue::nodes()` exposes an immutable nonempty postorder plan. A source
node carries its opaque identity. `PaletteNode::MixOklab` carries two earlier
node indices and the strictly interior fraction of its second operand. The last
node is the result. Style can materialize the plan in one forward pass,
performing each per-entry CSS Color interpolation in Oklab. Existing authored
mixes can remain opaque source operands with their own computation rules.
Nested samples preserve their operation boundaries; they are not flattened into
leaf weights. Merging interns identical operations so repeated branches share
subplans. Flat ownership also makes sampling, equality, debugging and destruction
independent of nesting depth, without a fixed depth cutoff.

Storage and index arithmetic are checked. `PaletteErrorKind::Capacity` reports a
plan that could not be represented, leaving its operands unchanged.
`InterpolationOutcome::PaletteFailure` and `InterpolationError::Palette` retain
its property, progress, complete endpoints and typed cause. Sampling reports a
failed result rather than substituting an endpoint for storage failure.

These symbolic rules follow [Fonts 4 §9.1](../../../references/css-fonts-4--WD-css-fonts-4-20260907--03626a0c8565.md#font-palette-prop)
and the [palette interpolation cases](https://github.com/web-platform-tests/wpt/blob/master/css/css-fonts/animations/font-palette-interpolation.html).
See the [module example](../src/palette.rs) and
[palette sampling tests](../tests/font_palette.rs). Style owns live parent/context
updates, chosen-font and palette-rule resolution, contextual equivalence,
percentage normalization inside supplied authored mixes and per-entry colors.
The sampler combines the supplied symbolic palette values and leaves
font-resource lookup and per-entry color computation to Style.

## Computed basic shapes

`shape::ShapeValue` combines a guarded `BasicShape` with an explicit semantic
`ReferenceBox`. The caller selects the applicable box, resolves SVG box aliases,
normalizes centers and polygon coordinates to top-left x/y offsets, and expands
inset offsets and corner radii. No used box dimensions or geometry revisions are
read or inferred by Animation. `InterpolableValue::shape(value)` enters the same
pair, transition, keyframe and native running-transition sampling paths as other
computed endpoints.

| Shape | Supplied computed components | Compatibility |
| --- | --- | --- |
| `Circle` | One `Radius`, normalized center | Both radii must be numeric. |
| `Ellipse` | Independent horizontal/vertical `Radius`, normalized center | All four endpoint radius branches must be numeric. |
| `Inset` | Four top/right/bottom/left offsets, eight corner radii | Already expanded fixed arrays; corners run top-left, top-right, bottom-right, bottom-left independently for each axis. |
| `Polygon` | Nonempty ordered vertices, effective fill rule, rounding length | Equal vertex counts and fill rules; rounding lengths interpolate. One and two vertices are valid. |
| `Shape` / `Path` | Initial point, ordered native commands, effective fill rule | Equal command kinds, forms, control counts and effective control anchors, with equal fill rules. |

`LengthPercentage::new` accepts only the existing computed Length, Percentage
and LengthPercentage numeric domains, including symbolic sources. Other numeric
roots are rejected with `ShapeErrorKind::InvalidNumericType`. Polygon rounding
must have the pure Length domain. The caller supplies a computed zero length
when rounding was omitted, and an effective fill rule when none was authored.
The records do not preserve optional authored grammar or calculate defaults.

Reference boxes, function kinds, polygon metadata and all radius keyword
branches are preflighted for the complete pair before any arithmetic. A conflict
switches the entire shape at progress 0.5. Circle/ellipse radius keywords remain
discrete even when identical; Animation does not invent their radius geometry.
Compatible pairs interpolate every numeric component through the computed
numeric owner, retaining percentages and contextual source identities. Exact
endpoints preserve the supplied complete shape, except compatible mixed
Shape/Path pairs always produce Shape. Negative sampled radii, inset
overlap, corner overlap and polygon-rounding geometry remain caller-owned.

Arithmetic failures retain a `ShapeError` identifying the component and its
underlying `ComputedError`. Polygon result-vertex storage uses checked reservation
and reports a separate `Capacity` error. `InterpolationOutcome::ShapeFailure` and
`InterpolationError::Shape` retain the property, progress and complete endpoints;
track sampling classifies the sample as `Failed`, and native running-transition
sampling returns a typed interpolation error. A later exact endpoint can recover.

These rules follow the pinned [Shapes 1 combination clause](../../../references/css-shapes-1--CRD-css-shapes-1-20250612--f7e528fad309.md#basic-shape-interpolation).
Shape addition and accumulation are outside this capability. Property animation
metadata and transition applicability remain caller policy. See [sampling a basic shape](how-to.md#sample-computed-basic-shapes).

### Native shape and path commands

`CommandShape::new` requires at least one trailing `ShapeCommand` after its
initial absolute point. `CommandPath::new` accepts an initial move alone and
requires all coordinate and radius operands to have the pure Length domain.
Path controls use `Origin` for absolute commands and `Start` for relative
commands. Rotation is guarded as a computed Angle. The supplying owner expands
SVG implicit parameter groups and treats an initial relative moveto as the
absolute initial point; subsequent implicit relative lines remain relative.
Animation does not parse CSS or SVG, resolve logical positions, or execute geometry.

Commands retain Move, Line, HorizontalLine, VerticalLine, Curve, Smooth, Arc and
Close identity. `Endpoint` and `AxisCommand` retain `To`/`By`; percentages in
relative displacements still use the reference box axes. Curves retain one or
two explicit controls. Smooth commands retain zero (quadratic) or one (cubic)
explicit control; reflection of implied controls remains with geometry. That
consumer reflects only within the matching SVG quadratic/cubic family, otherwise
using the segment start. Close resets that consumer's current point to the
current subpath's initial point. No derived controls or endpoints are evaluated
merely to sample explicit operands.

`ControlPoint` requires an effective `ControlAnchor`. Lower an absolute bare
position to Origin and an omitted relative anchor to Start. The selected Shapes
clause does not define conversion between different control anchors. This model
requires equal effective anchors, following the explicit no-interpolation cases
in [WPT's control-point tests](https://github.com/web-platform-tests/wpt/blob/master/css/css-masking/animations/clip-path-interpolation-shape-control-points.html).
Start, End and Origin remain distinct even when a particular box or segment
would make their geometry coincide.

The [pinned command interpolation clause](../../../references/css-shapes-1--CRD-css-shapes-1-20250612--f7e528fad309.md#interpolating-shape)
requires Path output only for two Path endpoints. Compatible mixed pairs return
Shape even at progress zero and one, reusing the complete endpoint's immutable
command data. Incompatible pairs instead switch the original complete value at
0.5, including its function kind. Preflight covers all commands before arithmetic.

Effective arc defaults are Ccw, Small and zero-angle rotation, supplied by the
caller. Differing sweep/size flags select Cw/Large only at strict interior
progress; at or below zero they retain the starting flag, and at or above one
the ending flag. Equal flags stay equal. This bounds the pin's unspecified
extrapolation behavior to the endpoint flags, matching
[WPT's negative and greater-than-one cases](https://github.com/web-platform-tests/wpt/blob/master/css/css-masking/animations/clip-path-interpolation-shape.html).
Angles interpolate numerically without shortest-angle folding.

Arc radii are supplied with their percentage bases already made suitable for
independent horizontal/vertical operands. One authored radius uses the box's
diagonal divided by sqrt(2) for its percentages; two radii use width and height.
To preserve a single radius without evaluating a box here, supply a pure-Length
symbolic source for the diagonal-based quantity in both axes, retaining its
source/context mapping. Do not copy a diagonal percentage into two ordinary axis
percentages. Geometry owns negative-radius absolute values, zero-radius lines,
coincident-endpoint suppression and enlargement of undersized arcs. Sampling
retains these operands unchanged except for their checked numeric interpolation.

Command storage uses checked reservation. Arithmetic errors identify the initial
point or the indexed command operand, retain complete endpoints through existing
shape diagnostics, and allow later successful samples. No command parser, shape
addition, accumulation or geometry engine is introduced.

## Computed Grid tracks

`grid::GridValue` represents template `None`, a nonempty `TemplateTrackList`,
a nonempty `ImplicitTrackList`, or the distinct Grid 2 `SubgridTrackList`.
CSS/Style supplies effective longhands and
validated, decoded custom identifiers. It expands the `grid`, `grid-template`,
`grid-row`, `grid-column` and `grid-area` shorthands. Template lists retain a
leading `LineNames` group and a trailing group after each `TrackSection`;
fixed and automatic repeat bodies preserve the same grouping. Animation preserves supplied
group order and case, without parsing or validating authored identifiers.

`TrackSize` preserves plain breadth, explicit `minmax()` and `fit-content()`
tags. `TrackBreadth` distinguishes length-percentage, flex, `auto`, `min-content`
and `max-content`. Numeric roles use the computed numeric owner; lengths and
percentages retain independent terms and contextual source identity. Flex is
typed separately from length and accepts a finite numeric factor. A `minmax()`
minimum cannot be flex. Signed intermediate numbers remain available for final
caller-owned range handling; no percentage basis or intrinsic geometry is used.

Equal outer lists combine corresponding items, without repeating either list to
a least common multiple. Unequal lists switch the whole value at progress 0.5.
Fixed repeats combine only when corresponding repeat sections have the same
positive integer count and body track count. Any incompatible repeat structure,
including a repeat opposite an ordinary track, switches the whole template.
Compatible lists combine numeric leaves and switch line-name groups or
incompatible leaf kinds locally. Two explicit `minmax()` functions combine their
minimum and maximum independently. A plain track opposite explicit `minmax()`,
or `fit-content()` opposite another function kind, switches that track item.

These rules use the pinned [Grid 2 property animation tables](../../../references/css-grid-2--CRD-css-grid-2-20250326--05aa64853c44.md#propdef-grid-template-columns)
and [repeat combination clause](../../../references/css-grid-2--CRD-css-grid-2-20250326--05aa64853c44.md#repeat-interpolation),
with the [CSSWG integer-repeat resolution](https://github.com/w3c/csswg-drafts/issues/3503)
and the explicit-minmax/local-item and incompatible-repeat whole-value cases in
the [WPT interpolation source](https://github.com/web-platform-tests/wpt/blob/master/css/css-grid/animation/grid-template-columns-interpolation.html).
The pinned repeat prose says that differing repeat structure combines
discretely, without specifying that surrounding tracks switch too; this API
selects the whole-value WPT behavior. The generic by-computed-value rule would
switch a whole incompatible `minmax()` record; this API instead selects WPT's
independent-bound behavior. These are explicit source determinations, not
claims of complete Grid or browser parity.

`FixedRepeat` requires a positive count and nonempty, nonnested body.
`AutomaticRepeat` retains `AutoFill` or `AutoFit`, a leading name group and a
nonempty fixed-size body. `TemplateTrackList::new` permits only one automatic
repeat and requires fixed-size ordinary tracks and fixed-repeat bodies around
it, following the [auto-track-list grammar](../../../references/css-grid-2--CRD-css-grid-2-20250326--05aa64853c44.md#track-sizing).
Fixed-size means a length-percentage breadth, or an explicit `minmax()` with at
least one length-percentage bound. A template containing automatic repeat always
switches as a complete original, including when the two automatic keywords are
identical. This follows the [restrictive CSSWG resolution](https://github.com/w3c/csswg-drafts/issues/3503#issuecomment-456901487),
which requires numeric equal repeat counts for smooth combination, and the WPT
automatic-repeat cases. The pin's literal equal-first-argument prose would also
allow equal automatic keywords; the numeric-only resolution is selected here.

`SubgridTrackList` retains the ordered `SubgridSection` sequence of line groups
and `NameRepeat` records, following the [distinct computed subgrid form](../../../references/css-grid-2--CRD-css-grid-2-20250326--05aa64853c44.md#computed-tracks).
Bare subgrid and empty individual line groups are valid; name-repeat bodies must
contain at least one group. Name-repeat permits a positive fixed count or
`AutoFill`, with at most one automatic name-repeat in the list. `AutoFit`, track
sizes and nested repeats are unrepresentable there. Subgrid values switch as
complete originals; no name-repeat count is expanded or interpolated.

`GridValue::add` and `accumulate` reuse the same structure traversal and computed
numeric owner. Matching numeric length-percentage/flex leaves, explicit minmax
bounds and fit-content limits combine componentwise. Accumulation uses Values'
[default of addition](../../../references/css-values-4--WD-css-values-4-20240312--7b6b68c34d00.md#accumulation).
Line-name groups take the effect's supplied ordered names. Incompatible ordinary
items or bounds take effect B locally; incompatible outer/repeat structure,
automatic-repeat templates and subgrid take the complete effect B before any
numeric work. Matching fixed-repeat counts remain unchanged. These choices
follow the [WPT Grid composition cases](https://github.com/web-platform-tests/wpt/blob/master/css/css-grid/animation/grid-template-columns-composition.html)
and Values' [non-additive fallback](../../../references/css-values-4--WD-css-values-4-20240312--7b6b68c34d00.md#addition).
No list least-common-multiple, concatenation or used percentage basis is applied.

Used repeat expansion, subgrid adoption, masonry and layout are outside this
native Grid 1/2 capability. Supplied
`grid-template-areas`, `grid-auto-flow` and placement longhands use the existing
discrete value path; Animation does not derive those values from track geometry.

`InterpolableValue::grid` enters pair, transition and keyframe sampling.
`InterpolationOutcome::GridFailure` and `InterpolationError::Grid` retain the
complete endpoints, property and progress. `GridError` identifies the outer
section, repeat item and numeric role when applicable. Result storage uses
checked reservation, and a failed sample publishes no interpolated prefix.
Track samples classify these failures as `Failed`; a later valid progress can
recover. Composition's numeric causes retain the Add/Accumulate operation and
indexed role. `AnimationTypeOperations::new(AnimationType::ByComputedValue)`
delegates Grid addition and accumulation to these owned operations when sampling an effect stack.
`PropertyOperationErrorKind::Grid` preserves the typed cause and its source
chain through animation/keyframe diagnostics. Property applicability and
transitionability remain consumer choices.

## Computed Masking values

`masking::ClipPathValue` retains `None`, an opaque `ClipSource`, a box-only
`shape::ReferenceBox`, or a complete `ShapeValue`. `ClipSource::new` rejects
blank text and control characters, preserving the caller's exact identity.
Its typed `ClipSourceError` identifies the invalid category and a control
character's byte index. Style/resources own URL syntax, absolutization, security
and availability; this guard neither parses URLs nor accesses resources.

`InterpolableValue::clip_path` enters the normal pair, keyframe and transition
samplers. Two shape branches delegate directly to `ShapeValue::interpolate`,
including numeric basic shapes, native Shape/Path commands, reference-box and
command compatibility, extrapolation and mixed-source output kind. Failure uses
the existing `ShapeInterpolationFailure`, retaining the complete clip-path
endpoints, property, eased progress and original typed shape diagnostic. A later
representable sample can recover. Every other branch pairing selects the whole
starting value below 0.5 and ending value at or above 0.5. No source is fetched,
box outline generated, or semantic reference box equated from used geometry.

An omitted shape reference box is supplied explicitly by the property's owner.
The shape sampler retains its mixed Shape/Path result kind at exact interpolation
endpoints. CSS transitions separately hold their supplied start value through
the delay boundary; subsequent interpolation delegates normally.

The selected [Masking property tables](../../../references/css-masking-1--CRD-css-masking-1-20210805--79ff953b4747.md)
make `mask-image`, `mask-mode`, `mask-repeat`, `mask-clip`, `mask-origin`,
`mask-composite`, `mask-border-source`, `mask-border-mode`, `mask-border-slice`,
`mask-border-width`, `mask-border-outset`, `mask-border-repeat`, `clip-rule`
and `mask-type` discrete. Callers select the existing complete discrete family
for these values. Backgrounds value-definition reuse does not change their
animation type, including numeric mask-border operands.

Legacy `clip` rectangles use four `ComputedValue` length-or-discrete-`auto`
components in top/right/bottom/left order. Matching `auto` remains symbolic;
other corresponding lengths interpolate, including negative offsets. One
`auto`/length conflict makes the complete rectangle discrete. Whole `auto` is a
separate single-component branch and also switches discretely against a rectangle.
The sampler never resolves an edge against border-box dimensions or aliases
`rect(auto, auto, auto, auto)` to whole `auto`.

Shorthands contribute through their effective longhands; `mask` also resets the
six mask-border longhands. The [shared layer sampler](#background-and-mask-layer-positions-and-sizes)
handles repeatable `mask-position` and `mask-size`. Style supplies computed layers,
resource identities and SVG context. Geometry and paint consumers apply the
samples. Runtime publishes composed results.

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

`css_animation::CssAnimationList` separately reconciles caller-resolved CSS
animation snapshots without mutating playback or publishing lifecycle events.

Root-supplied normalized tracks are the input shape for track sampling. Root/style
choose transition generation, keyframe lookup, CSS endpoint synthesis, property
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

This explicit premultiplied path continues to serve shadow and filter
composition. Consumers select their common operands and apply downstream
gamut mapping and final property range policy.

Permanent unsupported behavior is a stable unsupported outcome.
Recoverable sample-local arithmetic failure is a failed sample that can later recover.
Unsupported same-family interpolation, such as diagnostic transform markers or
composite interpolation, produces `SampleClassification::Unsupported` and
`SampleValue::Unsupported`. Finite arithmetic overflow during an otherwise
supported interpolation produces `SampleClassification::Failed` and
`SampleValue::Error`, and later endpoint samples can recover.

## Standard-space color sampling

`InterpolationPair::resolved_colors` accepts two resolved `surgeist_color::Color`
values and their selected `surgeist_color::InterpolationMethod`. The colors
may start in different spaces. `KeyframeSegment::from_pair` and
`TransitionTrack::from_pair` admit this pair with their normal timing and
offset validation. Construct one pair per interval when methods differ.
`InterpolationPair::color_endpoints` also accepts checked samples.
`InterpolableValue::color_endpoint` exposes either supported operand to a
transition property policy. Computed keyframes can use
`AnimationTypeOperations::for_color` with resolved or sampled endpoints.

Active sampling passes the finite eased progress unchanged. Color owns the
ordered conversion, missing-component carry and fill, hue arc, premultiplication
and unpremultiplication. A running sample is `InterpolableValue::ColorSample`
with its selected space, optional finite channels, and optional extended
alpha. At a delay hold, the sampled value is the held endpoint.
Arithmetic failures retain the property, eased progress, original endpoints
and typed Color cause through `ColorInterpolationFailure`. A later sample can
recover on the same track.

Style resolves contextual colors and selects the method. A running transition
can use its checked current sample as the replacement start. The delay hold
returns that exact sample. A negative delay advances the replacement's
creation-time sample under the existing timing rules. Reversal preserves
duration shortening.
`TransitionPropertyPolicy::equivalent` defaults to exact same-space comparison
between a checked sample and a resolved color when their numerical meanings
agree. Style can supply contextual equality. Checked color pair admission
requires an explicit method. Color addition and default accumulation select
the effect operand.

## Normalized filter lists and filter images

[`filter`](../src/filter.rs) owns `FilterInput`, immutable `FilterFunction`
samples and `FilterList`. Supply resolved CSS-pixel lengths, explicit colors,
`TransformAngle` values and typed number/percentage amounts. Percentage 1 means
100%. Input constructors reject nonfinite/negative sigma and amounts, invalid
endpoint alpha and empty URL reference tokens. Drop-shadow offsets can be
negative. The singleton `FilterDropShadow` has offsets, standard deviation and
color; no spread/inset slot exists. Neither it nor blur uses box-shadow blur
radius semantics. Style retains authored syntax, omitted-argument computation,
currentcolor, symbolic context and color conversion.

The [selected Filter Effects 1 rules](../../../references/filter-effects-1--WD-filter-effects-1-20181218--b73408da4068.md#interpolation-of-filter-functions)
interpolate blur as length, seven amount functions as numbers, hue rotation
without normalizing complete turns, and one drop shadow componentwise.
The shared checked scalar and premultiplied color kernels handle finite eased
progress, including extrapolation. Sampled lists can be reused as endpoints
without reapplying input guards. Each paired color must already use the same
explicit interpolation space. Used-value clamping remains with consumers:
amounts above one remain valid computed inputs, including grayscale, invert,
opacity and sepia whose effect use caps the amount at one. Filter-function pixel
execution in sRGB and SVG primitive color-space policy are distinct from this
supplied interpolation-space boundary.

`InterpolableValue::filter` enters ordinary pair, transition and keyframe
sampling. `FilterInterpolationFailure` retains the property, complete endpoints,
eased progress and indexed/component `FilterError`. No partial list is returned;
later finite samples can recover. Capacity reservations return typed errors.

Before arithmetic, interpolation checks the complete pair of lists for URL
references and mismatched corresponding function kinds. Either condition selects
the entire original from list below progress 0.5, or the entire to list otherwise.
For compatible lists, append missing identity functions at the end. Empty means
`none`; it receives the other list's corresponding identities. Interpolation
retains the padded shape, including at exact endpoints. Identities are blur zero,
hue zero, brightness/contrast/opacity/saturate one, grayscale/invert/sepia zero,
and a transparent zero-offset/zero-sigma drop shadow. These differ from omitted
arguments: all seven omitted amounts compute to one, and a missing drop-shadow
color comes from the color property. The caller supplies those computed defaults.
`added_to` concatenates underlying first; `accumulated_with` returns the entire
effect B under the [selected list algebra](../../../references/filter-effects-1--WD-filter-effects-1-20181218--b73408da4068.md#animation-of-filters).

[`filter::image`](../src/filter/image.rs) specializes native filter() image pairs.
Retain each complete original image in `FilterImage<I>`. For a filter() form,
supply its list and a `FilterImageSource` token certifying one immutable used
image plus all non-list context. Clone that token only for verified identical
facts, and issue a new token when the source, concrete object size or other
context changes. Keep the source-token-to-image mapping alive for sampled output.
URL text equality alone does not certify compatible sources.

`FilterImage::sample` returns a filtered source/list when both forms share that
verified source/context and numerical list interpolation succeeds. Otherwise it
executes the supplied generic image interpolation callback exactly once with both
complete originals and unchanged progress, preserving its typed result or error.
This includes URL/kind incompatibility and numerical/capability failure; it does
not replace generic image interpolation with a property-level discrete swap.
[Filter-image execution](../../../references/filter-effects-1--WD-filter-effects-1-20181218--b73408da4068.md#FilterCSSImageValue)
retains source dimensions/origin and uses duplicate edges for blur; those image
domain facts remain caller/renderer responsibilities. No decoding, fetching,
resource resolution, generic image algorithm or pixel execution is added here.

## Resolved shadow lists

[`shadow`](../src/shadow.rs) owns normalized box/text shadow inputs and sampled
lists. `ShadowGeometry` stores finite CSS-pixel X/Y offsets, blur radius and
spread in that order. `ShadowInput::new` checks nonnegative blur, nonnegative
spread for the selected Text Decoration 4 shape, and ordinary alpha in [0, 1].
Box spread can be negative. Both shapes permit inset. `ShadowList::new` requires
all inputs to match the list's explicit `ShadowKind`; an empty list is `none`.
The shared slice exposes immutable `Shadow` samples. These preserve finite
extended geometry and alpha during extrapolation, and the sampled list can be
reused as an endpoint without repeating authored range checks.

`InterpolableValue::shadow` enters the normal pair, keyframe and transition
samplers. Each paired color must already use the same explicit rectangular
interpolation space. Animation reuses its premultiplied color/scalar kernel;
it performs no currentcolor resolution, color conversion, missing-channel
processing, contextual length evaluation or final clamping.

Interpolation pairs shadows in order and appends transparent zero-geometry
blanks to the shorter list, using the corresponding real shadow's inset status
and color space. This is end padding, not repetition to a common multiple.
Compatible interpolation retains the padded list, including at exact endpoints.
An inset mismatch in any real pair switches the entire original list at 0.5.
These rules follow [Web Animations shadow-list combination](../../../references/web-animations-1--WD-web-animations-1-20230605--e814a799ef24.md#animating-shadow-lists).

`added_to` concatenates underlying first without inset matching.
`accumulated_with` uses the same padding and matching as interpolation, adds
geometry and selects the effect color: [Colors are not additive](../../../references/css-values-4--WD-css-values-4-20240312--7b6b68c34d00.md#combine-colors).
If the effect requires padding, its blank color is transparent. An accumulation
inset mismatch selects the complete effect value. Input-kind mismatches, color
space mismatches, capacity failures and nonfinite arithmetic return `ShadowError`
without partial output. `ShadowInterpolationFailure` retains the property,
complete endpoints, eased progress and indexed/component diagnostic; sampling
publishes a failed property, with later finite samples able to recover.

This is the shared box/text value algebra. Filter `drop-shadow()` has a distinct
singleton shape with standard deviation and no spread/inset; it is not admitted
as a box/text input. Filter-list composition, full Color interpolation, broad
background/decoration property lowering, and Style/Text/Render publication remain
with their respective owners.

## Resolved transforms

[`transform`](../src/transform.rs) owns `ResolvedTransformList`, `TransformFunction`,
`TransformMatrix`, `TransformMatrix3d`, `RotationAxis`, `CssPixels`, `ScaleFactor`,
`TransformAngle` and their numeric
diagnostics. Scalars and matrix coefficients are finite and immutable. Finite
negative lengths/scales, zero scales and singular matrices are valid. Matrices
use CSS column-major order: `[a,b,c,d,e,f]` for affine matrices, or 16
coefficients indexed as `column * 4 + row` for homogeneous matrices. Lengths are resolved CSS pixels;
angles have explicit radians, degrees or turns constructors and retain turns.
Function variants preserve CSS names and optional argument arity.

`TransformValue::resolved(list)` enters the normal interpolation, transition and
keyframe sampling paths. `resolved_list()` borrows the immutable payload.
Unsupported marker constructors never parse their text: even `resolved-2d` or `resolved-3d`
remains an unsupported marker. `kind()` is descriptive text; it does not certify
a concrete payload. Both endpoints must carry resolved lists for interpolation.

An empty list represents `none`. Sampling pads the shorter list with matching
identity functions. Functions with matching name/arity interpolate numerically;
translate/scale derivatives normalize to their common primitive, promoting to
3D when either operand is a 3D function. Different
skew axes have no shared primitive. At the first unmatched pair, each remaining
suffix is post-multiplied into a matrix and interpolated as one matrix. Matching
prefixes retain their function semantics, including full rotation turns.
`matrix()`, `matrix3d()` and `perspective()` pairs use decomposition even when
their names match. Matching rotate3d primitives normalize their axes: equal axes
or a zero angle retain direct angle interpolation; different nonzero axes use
one matrix pair while following matching functions remain separate. A zero
rotation axis is a valid no-op. Direct matching rotation angles retain turns.

Matrices that both have actual 2D coefficients use the pinned [Transforms 1 algorithm](../../../references/css-transforms-1--CR-css-transforms-1-20190214--4672e86844bf.md#interpolation-of-2d-matrices):
decompose translation, signed scales, angle and residual; normalize flipped axes;
adjust matrix rotations to the shorter arc; interpolate components and recompose.
[#552](https://github.com/bj-data-eng/surgeist/issues/552) selects direct placement
of extracted affine translation in recomposition. The pinned residual-weighted
translation does not reconstruct matrices with shear. The selected midpoint of
`matrix(1,0,1,1,10,20)` and `matrix(1,0,1,1,30,40)` is
`matrix(1,0,1,1,20,30)`, independently witnessed in Firefox 152 and corroborated
by [pinned WebKit affine recomposition](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/platform/graphics/transforms/TransformationMatrix.cpp#L1878-L1893).

Other matrices follow the pinned [Transforms 2 decomposition and recomposition](../../../references/css-transforms-2--WD-css-transforms-2-20211109--f1ada2510670.md#interpolation-of-3d-matrices).
Normalize by the homogeneous W coefficient; isolate perspective and translation;
extract signed scales, XY/XZ/YZ skew and a unit quaternion. Recompose in
perspective, translation, rotation, YZ/XZ/XY skew and scale order. Stable unit
rotation extraction preserves mixed-sign half-turn axes. The selected
column-major quaternion signs reconstruct positive quarter turns rather than
transposing them. These bounded source repairs are recorded in
[#552](https://github.com/bj-data-eng/surgeist/issues/552), with
[CSSWG #3230](https://github.com/w3c/csswg-drafts/issues/3230),
[CSSWG #3710](https://github.com/w3c/csswg-drafts/issues/3710) and
[pinned WebKit matrix signs](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/platform/graphics/transforms/TransformationMatrix.cpp#L640-L654).

Extracted quaternions use positive W whenever W is nonzero. At exact W=0,
the dominant vector component is positive, with X, then Y, then Z breaking
magnitude ties. Interpolation retains the pinned unadjusted SLERP arc; it does
not align endpoint signs to choose a shorter arc. The half-sum/half-difference
angular basis handles distinct near-antipodal or near-identical rotations even
when their dot product rounds to -1 or 1. Exactly equivalent rotations remain
constant. Thus spatial matrix Z rotations +170° and -170° pass through identity,
while direct matching rotation functions continue to interpolate their angles.

`perspective(none)` is identity under the selected §16 rule, superseding the
conflicting neutral-element note. Endpoint depths are floored to 1 CSS pixel
when matrices are formed. Reciprocal depth interpolates through the matrix's
perspective coefficients: none to 400px has coefficient -0.00125 halfway and
+0.00125 at progress -0.5. Finite extrapolated coefficients remain unrestricted.
See the [perspective resolution](https://github.com/w3c/csswg-drafts/issues/6488#issuecomment-896962025).

Exact singularity of finite matrix endpoints selects whole-value discrete
fallback: the original from value below progress `0.5`, the to value otherwise.
Determinant signs compare exact binary significand products, without an
epsilon or overflowing/underflowing floating determinant products. Spatial
decomposition additionally requires nonzero homogeneous W and an invertible
perspective base; failure selects the same discrete fallback. A singular
intermediate sample remains a value for caller-owned visibility. Column
directions are normalized at unit magnitude before restoring scale, so
rounded subnormal lengths cannot distort the other axis. An established singular
matrix pair selects fallback before arithmetic in a later unmatched suffix.
Scalar, decomposition, composition and recomposition overflow instead produce typed
sample-local arithmetic failure with a function index and transform component.
Exact zero/one progress returns the original concrete endpoint. Finite eased
progress outside the unit interval is preserved; output is never clipped.

`ResolvedTransformList::added_to` concatenates lists in operand order, retaining
all function identities. `to_matrix3d()` performs checked CSS post-multiplication.
`to_matrix()` returns an affine result or `TransformErrorKind::NotTwoDimensional`
when a spatial result cannot be represented without loss. Numeric failures use
`TransformErrorKind::NonFinite` and identify the component and list position.

`accumulated_with(effect)` combines an underlying list with an effect using the
same matching, padding and suffix rules. Direct parameters add, using a+b−1
for scale parameters. Checked compensated sums retain cancellation and identity
cases, including subnormal underlying values. Decomposed affine residual diagonal
and scale parameters also use one-based accumulation. Spatial scale and perspective W are one-based;
translation, skew and the other perspective components add. Decomposed rotations
use the underlying × effect quaternion product. A failed decomposition selects
the complete effect value; checked arithmetic failure returns an error without
publishing partial output. These choices follow the selected
[pinned quaternion accumulation](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/platform/graphics/transforms/Quaternion.cpp#L61-L73)
and [operand order](https://github.com/WebKit/WebKit/blob/73aa6c89e2cb77c46184a81aec944e4ab99d114d/Source/WebCore/platform/graphics/transforms/TransformationMatrix.cpp#L1801-L1828).

`neutral_for_addition()` preserves each function's kind and arity with the
pin-selected scalar-zero parameters: scales are zero, matrices are zero matrices,
rotation angles and translations are zero, and perspective is none. These values
are distinct from identity padding (scale one and identity matrices) and the
empty list used as the concatenation identity. Animation with a zero matrix
neutral selects discrete fallback because decomposition fails; neutral
perspective is the identity case described above.

Authored calculations, percentage/reference-box resolution, transform origins
and visibility decisions remain caller-owned. Callers resolve context before
construction and refresh the immutable endpoints when it changes; no percentage
basis is guessed.

## Capability table

| Capability | Status | Owner / behavior |
| --- | --- | --- |
| Time intake, finite durations, delays, and effective elapsed time | `supported` | `surgeist-animation` validates finite typed time values; runtime/root supplies elapsed time. |
| Signed nullable model time, immutable conversion snapshots and fixed-origin document time | `supported` | `surgeist-animation` owns pure coordinate arithmetic and checked document reports; callers supply clocks, activity and timeline identities. |
| Timing phases, fill modes, direction, iteration counts, play state, and next-frame hints | `supported` | `surgeist-animation` samples validated timing and reports stable or may-change hints. |
| Web Animations effect timing, signed local time, end delay, iteration start and relevance | `supported` | `EffectTiming` samples supplied animation observations through the shared phase/progress kernel without owning animation lifecycle or target queries. |
| CSS transition creation, cancellation, reversal, and completion decisions | `supported` | `transition::TransitionSnapshot` consumes coherent caller facts and returns immutable candidates and lifecycle outcomes. |
| CSS animation occurrence retention, start, and stop reconciliation | `supported` | `css_animation::CssAnimationList` consumes resolved rule/timing/applicability snapshots and preserves matched native identities and playback coordinates. |
| Typed native keyframe inputs and missing-offset computation | `supported` | `keyframe_input` normalizes sequences and property-indexed inputs while preserving specified offsets and caller payloads. |
| Computed keyframe effect sampling and native endpoint fallback | `supported` | `keyframe::ComputedKeyframes` selects sparse property intervals, preserves overlap branches, composes endpoints, and samples supplied property operations. |
| Cubic-bezier, steps, linear keyword, and CSS `linear()` easing | `supported` | `surgeist-animation` evaluates typed easing, including canonical linear control points. |
| Number, unrestricted percentage, and color interpolation | `supported` | Resolved colors delegate to Color with a selected method. The explicit premultiplied path retains its checked component algebra. |
| Normalized filter lists and native filter images | `supported` | Supplied resolved functions, identity end padding, URL/incompatible whole-value fallback, ordered composition and executed generic-image delegation. |
| Normalized box/text shadow lists | `supported` | Supplied resolved geometry and explicit colors, inset-aware end padding, whole-value discrete fallback, ordered addition and accumulation. |
| Discrete value switching | `supported` | `surgeist-animation` switches discrete values at the interpolation midpoint. |
| Computed numeric records, normalized positions, ratios and symbolic math | `supported` | Typed computed components combine without contextual evaluation or early clamping. |
| Computed Multi-column longhands | `supported` | Supplied computed endpoints, exact integer rounding, intrinsic and symbolic size sampling, selected Color methods and discrete switching. |
| Computed clip-path branches | `supported` | Supplied opaque sources, boxes and complete shapes retain whole-value fallback and typed shape failures. |
| Computed font numeric/discrete families and symbolic palette sampling | `supported` | Effective computed operands, fractional numeric samples, whole discrete selection and flat Oklab palette mix plans; Style resolves palette entries. |
| Computed circle, ellipse, inset and polygon interpolation | `supported` | Explicit reference boxes, guarded numeric domains, whole-shape compatibility and typed sample-local errors. |
| Native shape/path command interpolation | `supported` | Matching forms, effective anchors and fill rules; mixed pairs return Shape, with explicit arc-flag extrapolation. |
| Shape addition/accumulation | `deferred` | Composition operations remain outside the shape value model. |
| Resolved 2D/3D transform interpolation, list addition and accumulation | `supported` | Typed native lists, contextual primitive promotion, checked matrix decomposition, quaternion SLERP/accumulation and whole-value singular fallback. |
| Diagnostic transform markers and composite interpolation | `typed-unsupported` | Explicit markers retain unsupported outcomes; authored lowering and rendering are separate capabilities. |
| CSS parsing, cascade, transition generation, keyframe lookup, endpoint synthesis, property transitionability, allow-discrete policy, and shorthand/list expansion | `deferred/root-owned` | Root/style normalize authored declarations into this crate's typed tracks. |
| Runtime clocks, scheduling, lifecycle, invalidation, pause bookkeeping, and external input changes | `deferred/root-owned` | Runtime/root decide when to sample and pass `ElapsedTime` plus play state. |
| Display conversion, gamut mapping, clipping, and final property range clamping | `consumer-owned` | Consumers apply downstream output policy and provide common-space values for the explicit premultiplied path. Color owns conversion for resolved standard-space interpolation. |
| Root facade reexports, root integration tests, workspace wiring, and API artifact refresh | `root-owned` | These surfaces share the repository; inspect root source and current verification evidence for implemented integration. |

## Verification and attribution sources

[AGENTS.md](../AGENTS.md) lists local verification commands;
[How-to guides](how-to.md) describes running them.
[NOTICE.md](../NOTICE.md) records third-party source attribution coverage, while
[LICENSE](../LICENSE) contains the project's license.
