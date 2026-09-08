# Animation performance summary

This records the first optimization pass, measured on September 7, 2026.
The later [construction summary](construction-performance.md) describes the
subsequent pass. Raw run artifacts are no longer distributed with the repository.
These historical figures guide future measurements; they are not a browser-wide
performance claim.

## Compared revisions and method

- Original production: `6eb9c3f72d2e7d5722569da2eeee124327075acb`.
- Measurement foundation, with unchanged production:
  `8eac1f0faf2c7a1aaac920b053ea2700b5686fcd`.
- Optimized production: `ded7cd3df86b73a8998e76576ef17bc363ac60c8`.

Both revisions ran on an Apple M3 with 24 GiB RAM, aarch64 Darwin 25.6.0,
Rust 1.97.0, LLVM 22.1.6, and Cargo's default release profile. Each case used
10 ms warmup, 25 ms target batches, and seven batches. The figures below use
the second of two complete sequential baseline/candidate comparisons.

The throughput harness uses standard-library timing and `black_box`. Allocation
counts come from a separate single-threaded executable. Complete construction
includes caller inputs and disposal; prepared construction excludes input cloning
but includes grouping, validation, and disposal. Sampling starts with prepared
fixtures. Neither measurement includes CSS parsing, style resolution, scheduling,
layout, or painting.

The comparison covered 84 unchanged cases: grouped and interleaved tracks with
1/8/64 properties and 1/8/64 segments per property, seven value families,
piecewise-linear easing with 2/16/256 points, and cubic keywords and flat curves.
Another 25 cases measured the newly added `sample_into()` API separately.

## Sampling and easing

| Operation | Original | Optimized | Speedup |
| --- | ---: | ---: | ---: |
| Numeric transition sample | 32.64 ns | 15.17 ns | 2.15× |
| Percentage transition sample | 32.99 ns | 14.44 ns | 2.28× |
| Color transition sample | 35.51 ns | 14.88 ns | 2.39× |
| Discrete transition sample | 55.15 ns | 14.00 ns | 3.94× |
| Unsupported transform sample | 76.02 ns | 16.43 ns | 4.63× |
| Arithmetic-failure sample | 79.17 ns | 17.34 ns | 4.57× |
| Cubic `ease` | 239.54 ns | 44.96 ns | 5.33× |
| Cubic `ease-in-out` | 237.45 ns | 32.66 ns | 7.27× |
| Linear easing, 256 points | 133.79 ns | 13.16 ns | 10.16× |
| Grouped keyframe sample, 64 properties × 64 segments | 525.42 µs | 2.20 µs | 239.10× |
| Interleaved keyframe sample, 64 properties × 64 segments | 575.29 µs | 1.66 µs | 345.81× |

Keyframe sampling improved about 2–346× across the measured track shapes.
The largest gains belong to 4,096-segment stress fixtures. Cached property
groups and binary segment selection remove repeated discovery and linear scans.
Cubic easing improved about 4–7× on these curves; two-point linear easing
improved only about 3% in the second comparison.

The reusable output API's main benefit is its allocation contract. It was not
uniformly faster than allocating `sample()` in every measured case.

## Setup and ownership tradeoffs

| Operation | Original | First optimization |
| --- | ---: | ---: |
| Numeric transition construction | 30.60 ns | 73.86 ns |
| Discrete transition construction | 73.37 ns | 180.26 ns |
| Complete grouped 1-property/1-segment track | 189.96 ns | 261.29 ns |
| Complete grouped 64-property/64-segment track | 583.94 µs | 485.44 µs |
| Prepared grouped 64-property/64-segment track | 395.21 µs | 84.57 µs |

First-pass complete construction for one or eight properties was 29–54% slower
in the second comparison. Transition construction was 104–146% slower. These
setup costs motivated the subsequent construction pass; this table does not
describe the latest keyframe constructor.

Shared property/marker text and shared endpoints make owned samples and complete
diagnostics inexpensive to retain and clone. With legacy borrowed-text intake,
numeric transition construction rose from one allocation to three. Current
`from_text()` and `from_shared()` entry points, and reusing a validated property
key across segments, reduce text-intake costs; see the
[usage guide](how-to.md#prepare-property-keys-and-value-text).

Observed recurring transition allocations fell from one per numeric/percentage/
color sample, two per discrete sample, and three per unsupported or arithmetic
failure sample to zero. Sufficiently provisioned `sample_into()` also performs
zero allocations and reallocations. Allocating keyframe `sample()` retains an
output-vector allocation. Retained results can extend shared storage lifetimes.

## Reassessment

The [measurement procedure](how-to.md#measure-sampling-performance) documents the
retained throughput and allocation tools. Offline commands require the existing
dev-only `stats_alloc` dependency in the Cargo cache; the library has no runtime
dependencies. Behavioral and allocation checks remain executable in the crate.

These timings came from one host without CPU pinning or frequency control.
Allocation counts do not measure fragmentation, peak memory, or contention.
The later study used a different, symmetrically applied compiled timing boundary,
so its median improvements must not be multiplied into an original-to-current
lifecycle percentage. That complete lifecycle comparison has not been measured.

Keep the current architecture until upstream CSS workload profiling identifies
a further optimization need. New performance decisions require fresh paired
measurements with matching fixtures, toolchain, warmup, and batching.
