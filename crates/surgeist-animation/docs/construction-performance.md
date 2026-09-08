# CSS animation construction performance

This historical summary compares production baseline
`cfa6ea57adc6edf3ee6420055c1583552ee15769` with the selected implementation at
`ec885a71c4e16abb2a321e43d5fe4ef8f7588b12`. The baseline already includes the
earlier [sampling and easing improvements](performance.md); these percentages
are additional gains, not comparisons with the original engine.

Raw run artifacts are no longer distributed with the repository. The benchmark
and allocation executable remain available through the
[measurement procedure](how-to.md#measure-sampling-performance).

## Measurement scope

Measurements used one Apple M3 host with 24 GiB memory and Rust 1.97.0 in release
mode on September 7, 2026. Paired runs used identical fixtures, a 10 ms warmup,
seven 25 ms target batches, and alternating baseline/candidate execution order.
Both variants isolated the timed operation loop from the case driver with the
same `#[inline(never)]` helper. Allocation instrumentation ran separately.

The 1,693 comparable cases included existing construction, sampling, and easing
fixtures, 1,480 complete lifecycles, and 104 clone-and-drop cases. An additional
36 API-adoption cases were measured separately from unchanged legacy calls.

Lifecycles included caller input preparation, construction, 0/1/4/12/60 samples,
and destruction, using a 200 ms duration with linear or `ease` timing. Timestamps
were prepared outside timing: a midpoint for one sample, or evenly spaced times
including both endpoints for multiple samples. Transitions covered numeric,
percentage, color, and discrete values. Numeric keyframes covered 1/2/8
properties by 1/2/8/64 segments per property, grouped and interleaved, with fresh
keys per segment or one validated key per property cloned across segments.

The three output paths measured allocating `sample()`, `sample_into()` with
buffer creation and destruction inside the lifecycle, and `sample_into()` with
capacity provisioned outside timing. Persistent-buffer runs cleared retained
results inside each lifecycle. Clone runs prepared the original outside timing
and included both cloning and destruction.

## Historical results

Ratios are selected time divided by baseline time; lower is faster. Each fixture
uses its seven-batch median. Group medians are unweighted medians of those
ratios, averaging the two middle ratios for even groups. They are not aggregate
application timings.

| Workload | Cases | Median selected / baseline |
| --- | ---: | ---: |
| Keyframe construction | 36 | 0.787 |
| Transition construction | 7 | 0.993 |
| Transition sampling | 7 | 0.956 |
| Keyframe `sample()` | 25 | 0.972 |
| Keyframe `sample_into()` | 25 | 0.885 |
| Transition lifecycles | 40 | 1.000 |
| Keyframe lifecycles | 1,440 | 0.860 |
| Clone and drop | 104 | 0.662 |
| Easing | 9 | 1.000 |

These correspond to approximately 21% less keyframe construction time, 14% less
keyframe lifecycle time, and 34% less clone-and-drop time at the median fixture.
All lifecycle cases stayed within the initial 5% regression threshold. Three
other cases crossed it and received three focused paired comparisons each.
None exceeded 5% in at least two of those three pairs, the declared criterion
for a repeatable regression.

## Allocation and ownership

Single-property tracks validate and sample their original segment slice without
allocating lookup structures. Grouped tracks retain contiguous membership as
ranges and use index vectors for scattered membership, preserving original
segment order and first-appearance property order.

The following counts measure only `KeyframeTrack::new`: every caller input,
including the ID and segments, is prepared before the region. The snapshot
precedes track disposal; temporary grouping-map destruction is included.

| Layout / properties / segments per property | Baseline allocations / reallocations | Selected allocations / reallocations |
| --- | ---: | ---: |
| Single property / 1 segment | 3 / 0 | 0 / 0 |
| Single property / 64 segments | 3 / 4 | 0 / 0 |
| Grouped / 2 / 64 | 4 / 8 | 1 / 0 |
| Grouped / 8 / 64 | 12 / 33 | 2 / 1 |
| Grouped / 64 / 64 | 71 / 260 | 5 / 4 |
| Interleaved / 8 / 64 | 12 / 33 | 10 / 33 |

All 32 comparable constructor cases used fewer allocations, with no allocation
or reallocation increases. Requested owned-storage totals, including inline
owner roots, decreased or stayed equal in all 1,380 reported memory rows.
These totals exclude allocator overhead and do not represent process memory.

Valid borrowed-text intake uses one allocation; valid shared-text intake uses
none. Construct a `PropertyKey` once and clone it across segments. `from_text()`
avoids an intermediate `String`; `from_shared()` consumes an existing `Arc<str>`.
Invalid borrowed property/discrete text is rejected before allocation, while
blank unsupported markers may allocate their fallback. Text intake validates
nonblank content; CSS parsing and property support policy belong to the adapter.

Interpolation pairs retain shared immutable endpoints, costing one allocation
per pair during construction. This keeps owned values compact and makes endpoint
sharing and complete arithmetic diagnostics inexpensive. Results remain valid
after their originating track is dropped. Transition sampling and sufficiently
provisioned `sample_into()` retain zero allocations and reallocations, including
unsupported and arithmetic-failure results.

## Limits and future work

The selected implementation passed 240 tests, including four doctests, plus
offline checks, all-target Clippy with unsafe forbidden and warnings denied,
documentation generation, formatting, and allocation contracts.

These are historical measurements from one host and a synthetic fixture matrix,
not browser performance results. The original engine was not run through this
complete lifecycle matrix, and results should not be multiplied by gains from
the earlier study to claim a combined application speedup. With raw artifacts
removed, reproducing numerical comparisons requires new paired measurements.

The current storage and text-intake design remains in place. Further
optimization should follow profiles from upstream CSS integration, where actual
property reuse, track lifetimes, sampling frequency, and ownership patterns can
establish whether setup costs justify another design change.
