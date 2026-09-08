# Getting started

Sample a normalized opacity transition and observe an active sample.

## Prerequisites

Use a local checkout and an already-installed Rust toolchain supporting edition
2024, with Cargo. The [manifest](../Cargo.toml) declares no runtime dependencies
and no explicit minimum Rust version. Local verification also requires the
dev-only `stats_alloc` dependency in Cargo's cache. On a new checkout, populate
that cache with `cargo fetch` when dependency retrieval is authorized. The
commands below run from the crate directory and use offline mode.

## First sample

1. Run the public transition check shown in the [README](../README.md). It
   verifies a delayed transition before its active interval and should report
   one passing test.
2. In code consuming the library, construct and sample a normalized track:

Transition tracks are already normalized when they reach this crate:

```rust
use surgeist_animation::{
    AnimationDelay, AnimationDuration, AnimationPlayState, Easing, ElapsedTime,
    InterpolableNumber, InterpolableValue, PropertyKey, SampleClassification, TransitionTrack,
    TransitionTrackId,
};

let track = TransitionTrack::new(
    TransitionTrackId::new(1),
    PropertyKey::new("opacity").unwrap(),
    InterpolableValue::number(InterpolableNumber::new(0.0).unwrap()),
    InterpolableValue::number(InterpolableNumber::new(1.0).unwrap()),
    AnimationDuration::from_secs(1.0).unwrap(),
    AnimationDelay::from_secs(0.25).unwrap(),
    Easing::linear(),
)
.unwrap();

let sample = track.sample(
    ElapsedTime::from_secs(0.5).unwrap(),
    AnimationPlayState::Running,
);
assert_eq!(sample.classification(), SampleClassification::Active);
```

The assertion verifies an active sample at 0.5 seconds of effective elapsed
time: the 0.25-second delay has elapsed, leaving 0.25 seconds of active time.
Root supplies the resolved property and endpoints; the caller supplies elapsed
time and play state.

Continue with [keyframe sampling](how-to.md) or consult the
[public contract reference](reference.md).
