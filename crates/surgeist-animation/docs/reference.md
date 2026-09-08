# Reference

## Package and public interface

[Cargo.toml](../Cargo.toml) declares `surgeist-animation` 0.1.0, Rust edition 2024,
and the library import name `surgeist_animation`. It declares no runtime
dependencies, features, or explicit minimum Rust version. The dev-only
`stats_alloc` dependency instruments allocation measurements.

[src/lib.rs](../src/lib.rs) is the public front door. It reexports the contracts
from the public modules `diagnostics`, `easing`, `interpolation`, `keyframe`,
`progress`, `sample`, `time`, `timing`, and `transition`. The crate forbids unsafe
code. Tests live in source modules and the external consumer targets under
[tests/](../tests/); crate-level examples are doctests.

## Sampling contracts

`KeyframeTrack::sample_into(elapsed, play_state, &mut output)` replaces an
existing `Vec<SampledPropertyResult>` with one owned result per property, in
first-appearance order. It retains the buffer's capacity and allocates nothing
during sampling when capacity is sufficient, including unsupported and failed
results. `sample()` returns the same ordered results in a newly allocated vector.
Tracks retain their original `segments()` order independently of their lookup
index. Sampling remains stateless; retained results do not borrow their track.

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

## Integration boundary

Use the crate root reexports for public API access. The final front door exposes
typed time, progress, easing, timing, interpolation, transition, keyframe,
sample, and diagnostic contracts without requiring private module paths.

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

Canonical linear control points model CSS `linear()` easing with finite typed
points, duplicate input selection, interpolation, and extrapolation. Easing can
also be evaluated with unrestricted finite `EasingInput` values.

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
| Timing phases, fill modes, direction, iteration counts, play state, and next-frame hints | `supported` | `surgeist-animation` samples validated timing and reports stable or may-change hints. |
| Cubic-bezier, steps, linear keyword, and CSS `linear()` easing | `supported` | `surgeist-animation` evaluates typed easing, including canonical linear control points. |
| Number, unrestricted percentage, and explicit premultiplied color interpolation | `supported` | `surgeist-animation` interpolates finite values with checked arithmetic and explicit color spaces. |
| Discrete value switching | `supported` | `surgeist-animation` switches discrete values at the interpolation midpoint. |
| Transform and composite interpolation | `typed-unsupported` | Values are represented and produce typed unsupported samples rather than fallback values. |
| CSS parsing, cascade, transition generation, keyframe lookup, endpoint synthesis, property transitionability, allow-discrete policy, and shorthand/list expansion | `deferred/root-owned` | Root/style normalize authored declarations into this crate's typed tracks. |
| Runtime clocks, scheduling, lifecycle, invalidation, pause bookkeeping, and external input changes | `deferred/root-owned` | Runtime/root decide when to sample and pass `ElapsedTime` plus play state. |
| Color-space conversion, gamut mapping, clipping, and final property range clamping | `root-owned` | Root/consumers prepare interpolation-space values and apply downstream output policy. |
| Root facade reexports, root integration tests, workspace wiring, and API artifact refresh | `root-owned` | These surfaces share the repository; inspect root source and current verification evidence for implemented integration. |

## Verification and attribution sources

[AGENTS.md](../AGENTS.md) lists local verification commands;
[How-to guides](how-to.md) describes running them.
[NOTICE.md](../NOTICE.md) records third-party source attribution coverage, while
[LICENSE](../LICENSE) contains the project's license.
