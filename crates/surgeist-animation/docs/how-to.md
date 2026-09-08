# How-to guides

These procedures assume the [first transition sample](getting-started.md) is
working. Run commands from the crate directory with installed tooling.

## Sample a keyframe animation

Prepare resolved property values and contiguous segments covering offsets 0
through 1 for each property. Root/style own name lookup and endpoint synthesis.
Construct timing, a segment, and a track, then sample it:

```rust
use surgeist_animation::{
    AnimationDelay, AnimationDirection, AnimationDuration, AnimationPlayState, Easing,
    ElapsedTime, FillMode, InterpolableNumber, InterpolableValue, IterationCount,
    KeyframeAnimationId, KeyframeSegment, KeyframeTiming, KeyframeTrack, PropertyKey,
    SampleClassification, UnitRatio,
};

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
let surgeist_animation::SampleValue::Value(value) = output[0].value() else {
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
