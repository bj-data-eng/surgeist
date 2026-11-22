# How-to guides

These procedures assume the [first transition sample](getting-started.md) is
working. Run commands from the crate directory with installed tooling.

## Sample a keyframe animation

Prepare resolved property values and contiguous segments covering offsets 0
through 1 for each property. Root/style own name lookup and endpoint synthesis.
Construct timing, a segment, and a track, then sample it:

```rust
use surgeist_animation::easing::Easing;
use surgeist_animation::interpolation::{InterpolableNumber, InterpolableValue, PropertyKey};
use surgeist_animation::keyframe::{
    KeyframeAnimationId, KeyframeSegment, KeyframeTiming, KeyframeTrack,
};
use surgeist_animation::progress::UnitRatio;
use surgeist_animation::sample::SampleClassification;
use surgeist_animation::time::{AnimationDelay, AnimationDuration, ElapsedTime};
use surgeist_animation::timing::{AnimationDirection, AnimationPlayState, FillMode, IterationCount};

let timing = KeyframeTiming::try_new(
    AnimationDelay::from_secs(0.0).unwrap(),
    AnimationDuration::from_secs(1.0).unwrap(),
    IterationCount::finite(1.0).unwrap(),
    AnimationDirection::Normal,
    FillMode::Both,
)
.unwrap();
let segment = KeyframeSegment::new(
    PropertyKey::new("opacity").unwrap(),
    UnitRatio::new(0.0).unwrap(),
    UnitRatio::new(1.0).unwrap(),
    InterpolableValue::number(InterpolableNumber::new(0.0).unwrap()),
    InterpolableValue::number(InterpolableNumber::new(1.0).unwrap()),
    Easing::linear(),
)
.unwrap();
let track = KeyframeTrack::new(
    KeyframeAnimationId::new("fade").unwrap(),
    timing,
    vec![segment],
)
.unwrap();

let samples = track.sample(
    ElapsedTime::from_secs(0.5).unwrap(),
    AnimationPlayState::Running,
);
assert_eq!(samples[0].classification(), SampleClassification::Active);
```

The assertion verifies an active sample halfway through a one-second animation.
Each property produces one sample. Use the sample's classification, value,
completion status, and next-frame hint together; see the [reference](reference.md).

## Combine computed property values

Supply computed canonical units and retain percentages until their caller-owned
context can resolve them. For a length-percentage property, 10px to 50% at the
midpoint produces 5px plus 25%:

```rust
use surgeist_animation::computed::{
    ComputedComponent, ComputedDimension, ComputedNumeric, ComputedValue,
    DimensionKind, NumericLiteral,
};
use surgeist_animation::interpolation::{InterpolableValue, InterpolationProgress};

let length = ComputedNumeric::dimension(
    ComputedDimension::new(DimensionKind::Length, 10.0).unwrap(),
);
let percentage = ComputedNumeric::percentage(0.5).unwrap();
let from = ComputedValue::single(ComputedComponent::Numeric(length));
let to = ComputedValue::single(ComputedComponent::Numeric(percentage));
let result = from.interpolate(&to, InterpolationProgress::new(0.5).unwrap()).unwrap();
let ComputedComponent::Numeric(numeric) = &result.components()[0] else {
    panic!("expected computed numeric component");
};
let Some(NumericLiteral::Mix(mix)) = numeric.as_literal() else {
    panic!("expected independent length and percentage terms");
};
assert_eq!(mix.dimension().value(), 5.0);
assert_eq!(mix.percentage(), 0.25);
let sampled_value = InterpolableValue::computed(result);
```

Use these wrapped records as ordinary transition/keyframe endpoints. Use `add`
or `accumulate` to combine an underlying record with an effect's endpoint before
interpolation. Keep intermediate negative values: clamp only the final composed
result to the target property's range. Differing discrete components switch the
whole record; they do not leave interpolated numeric components behind.

For contextual math, create a `MathContext` for the complete immutable resolution
context and a distinct `source(type)` for each computed calculation. Retain a
caller-owned map from those handles to the original calculations. A
`ComputedNumeric::source(handle)` participates in the same operations, producing
the typed postorder plan described in the [reference](reference.md#computed-numeric-combination).

## Sample computed basic shapes

Prepare a `shape::ShapeValue` with an explicit already-selected `ReferenceBox`.
Construct the desired `BasicShape` from validated `LengthPercentage` values and
normalized `computed::ComputedPosition` coordinates. The
[module example](../src/shape.rs) constructs a centered circle and checks its
midpoint radius.

For transition/keyframe sampling, wrap each endpoint with
`InterpolableValue::shape(shape_value)` and use the ordinary track constructors
shown above. For an inset, supply exactly four expanded offsets and four
horizontal plus four vertical corner radii. For a polygon, supply a nonempty
vertex list, effective fill rule and computed rounding length; use an explicit
zero length when authored rounding was omitted.

Keep the caller's numeric source map alive for symbolic endpoints and results.
Do not resolve percentages just to construct these values. After sampling,
inspect the complete shape or the typed `InterpolationError::Shape` diagnostic.
Only the final consuming owner applies range/overlap constraints and executes
geometry. Different boxes, function kinds, radius keywords or polygon metadata
produce whole-shape discrete fallback. The [reference](reference.md#computed-basic-shapes)
distinguishes these pair rules from caller-owned transition applicability.

For native `shape()` and `path()` values, construct `CommandShape` or
`CommandPath` with an initial point, ordered `ShapeCommand` values and the
effective fill rule. Supply explicit control anchors and arc defaults. Path
admission requires pure pixel-length coordinate/radius operands and SVG control
origins. Lower authored single-radius percentages to a shared pure-Length
symbolic source with the diagonal-based context before supplying two arc radii.
Keep smooth controls implicit and command forms intact. See the
[native command contract](reference.md#native-shape-and-path-commands) for result
kind, control-anchor compatibility and extrapolated arc flags.

## Reuse keyframe sample storage

Keep one output buffer across samples. Given the `track` above:

```rust
let mut output = Vec::with_capacity(1);
track.sample_into(
    ElapsedTime::from_secs(0.25).unwrap(),
    AnimationPlayState::Running,
    &mut output,
);
track.sample_into(
    ElapsedTime::from_secs(0.5).unwrap(),
    AnimationPlayState::Running,
    &mut output,
);
assert_eq!(output.len(), 1);
let surgeist_animation::sample::SampleValue::Value(value) = output[0].value() else {
    panic!("expected sampled opacity");
};
assert_eq!(value.property().as_str(), "opacity");
let InterpolableValue::Number(opacity) = value.value() else {
    panic!("expected numeric opacity");
};
assert_eq!(opacity.value(), 0.5);
```

Each call replaces the previous results. Reserve enough capacity for the number
of distinct properties to avoid output allocations. Results own their values
and diagnostics and may be moved out or retained after dropping the track.

## Validate linear easing coordinates

Pass raw coordinates to `LinearEasing::new`, then inspect its validated points:

```rust
use surgeist_animation::easing::{LinearControlPointInput, LinearEasing};
use surgeist_animation::progress::NormalizedProgress;

let easing = LinearEasing::new(vec![
    LinearControlPointInput { input: 0.0, output: 0.0 },
    LinearControlPointInput { input: 0.5, output: 0.25 },
    LinearControlPointInput { input: 1.0, output: 1.0 },
])
.unwrap();
assert_eq!(easing.points().len(), 3);
assert_eq!(
    easing.evaluate(NormalizedProgress::new(0.25).unwrap()).value(),
    0.125,
);
```

Construction rejects empty input, nonfinite coordinates, decreasing input
coordinates, and unrepresentable normalized boundary outputs with typed errors.
Coordinate diagnostics identify the original list index and component. The
published points preserve finite coordinates exactly, including duplicate inputs
and signed zero; they have no public unchecked constructor or writable fields.

## Prepare property keys and value text

When preparing multiple segments for one CSS property, construct a `PropertyKey`
once and pass its clones to `KeyframeSegment::new`. Clones share immutable text
without allocating. A final segment can consume the original key. Independently
constructed keys with equal contents still identify the same property.

Use `PropertyKey::from_text("opacity")` for borrowed text, or
`PropertyKey::from_shared(name)` when the caller already owns an `Arc<str>`.
`DiscreteValue` offers the same two routes. Transform and composite markers offer
`unsupported_from_text` and `unsupported_from_shared`. Valid borrowed intake
allocates its shared storage once; valid shared intake consumes the handle
without allocating. Existing constructors remain available.

Keep all segments for a property adjacent in the input vector when convenient.
Such groups need no segment-index vector, and a single-property track needs no
grouping metadata allocations. Interleaved input remains supported; `segments()`
always retains the supplied order. These choices concern leaf input preparation;
actual CSS lowering and caller changes remain root-owned.

Exactly two properties share immutable lookup metadata across track clones.
Larger tracks own their group vector; the builder initially reserves four group
slots and eight temporary map entries. These private capacities do not limit
property counts or change the input contract.

## Verify crate-local work

Run these before handing off crate-local animation work:

Offline verification and measurement commands require the existing dev-only
`stats_alloc` dependency to be cached. The crate has no runtime dependencies.

```sh
cargo test --offline -p surgeist-animation
cargo test --offline -p surgeist-animation --doc
cargo doc --offline -p surgeist-animation --no-deps
cargo clippy --offline -p surgeist-animation --all-targets -- -F unsafe-code -D warnings
cargo fmt -p surgeist-animation --check
```

Each command must exit successfully. Tests exercise the crate and its public
examples; documentation generation checks the API docs; Clippy and formatting
check the Rust targets. The [agent guide](../AGENTS.md) owns the command inventory.

## Measure sampling performance

Use an optimized build and run throughput and allocation measurements separately:

```sh
cargo bench --offline -p surgeist-animation --bench sampling
cargo run --offline --release -p surgeist-animation --example allocation_check -- --report-only
```

The benchmark prints CSV with per-case median, minimum, and maximum times across
repeated batches. It measures complete construction, track construction from
prepared segments, and steady-state sampling. Construction timings include
disposal; prepared-input cloning happens outside the timed interval. Keep the
toolchain, machine, inputs, warmup, and batch settings identical when comparing
revisions. Use `--filter SUBSTRING` to select a case, or `--help` for batch controls.

The allocation executable uses a separate instrumented system allocator. It
reports each constructor or sampling boundary separately; it is not a throughput
benchmark.
Its default report-only mode supports measuring unoptimized revisions without
requiring optimized allocation counts. Both tools reject filters matching no case.

The added `lifecycle/` rows include complete construction, 0/1/4/12/60 samples,
and destruction for 200 ms animations. They distinguish fresh and reused
property keys, allocating output, a fresh reusable buffer, and a persistent
buffer. `clone/` rows measure clone plus destruction. `adoption/` rows measure
the new text APIs separately from unchanged legacy call sites.

Inspect constructor allocations and requested live storage with:

```sh
cargo run --offline --release -p surgeist-animation --example allocation_check -- --report-only --filter construct/track_only/
cargo run --offline --release -p surgeist-animation --example allocation_check -- --report-memory
cargo run --offline --release -p surgeist-animation --example allocation_check -- --check-construction
cargo run --offline --release -p surgeist-animation --example allocation_check -- --check-intake
```

`track_only` prepares the ID, timing, and segments outside the measured
constructor. `--check-construction` requires zero allocations and reallocations
for single-property tracks at that boundary. `--check-intake` enforces the new
text-route contracts while reporting legacy and blank-marker fallback counts
without fixing them in place.
Memory reports create complete inputs within each region and add inline root
sizes to net requested heap bytes. They cover tracks, retained results, and
cloned tracks; they do not measure allocator overhead, fragmentation, or RSS.

Enforce the steady-state allocation contracts with:

```sh
cargo run --offline --release -p surgeist-animation --example allocation_check -- --check-values
cargo run --offline --release -p surgeist-animation --example allocation_check -- --check-failures
cargo run --offline --release -p surgeist-animation --example allocation_check -- --check-reuse
```

The first command checks successful and unsupported transition results, the
second checks arithmetic failures, and the third checks reusable keyframe
outputs. All require zero allocations and reallocations in the measured loops.
These are separate from ordinary Cargo tests because allocation counts use a
single-threaded executable. See the [performance summary](performance.md) for
the original sampling study and [construction summary](construction-performance.md)
for the lifecycle study, baseline revisions, results, and limitations.

## Refresh attribution after changing shipped material

Inspect the manifest and the material included in the source distribution.
Update [NOTICE.md](../NOTICE.md) for new dependencies or bundled/adapted code,
and include the exact applicable upstream license and notice files under
`licenses/` when needed. Link each dependency entry to its local legal material
and verified upstream homepage. Keep the project's own [LICENSE](../LICENSE)
separate. Verify the notice and linked files are included together in the
package; builds should not retrieve legal material.
