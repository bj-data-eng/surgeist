# Boa v0.22 baseline

The reference source is the unmodified Boa release at
`337a3668a0dc86dd401ea20906e782249a64a228`. Its Test262 configuration
pins `d86b2294eb0a17eaa281ff12c73c473ec864c72f`. The execution host was
an Apple M3, Mac15,12, with 24 GiB RAM, macOS 27.2 (26B123),
`aarch64-apple-darwin`, rustc 1.99.1 (`58cf8f9c3`), and LLVM 23.1.1.
The runs used the pinned Cargo lock with no extra Rust compiler flags
and one Cargo build job.

## Rust suites

The [per-test Rust results](rust-tests-v0.22.log.gz) retain 1,953 passes,
four ignores, and zero failures across the selected upstream packages
and their documentation tests. Run from the upstream checkout.

```sh
cargo test --offline --locked -j 1 --profile ci \
    -p boa_ast -p boa_engine -p boa_gc -p boa_icu_provider -p boa_interner \
    -p boa_macros -p boa_parser -p boa_string -p boa_runtime -p boa_wintertc \
    -p tag_ptr -p small_btree -p boa_macros_tests -p insta-bytecode -p boa_tester \
    --features boa_engine/annex-b,boa_engine/intl_bundled,boa_engine/experimental,boa_engine/embedded_lz4
```

## Conformance

Run from the upstream checkout with the runner's default features, default
ECMAScript Next edition, default parallel execution and optimization disabled.

```sh
test262_revision=d86b2294eb0a17eaa281ff12c73c473ec864c72f
test "$(git -C "$TEST262_CHECKOUT" rev-parse HEAD)" = "$test262_revision" &&
    test -z "$(git -C "$TEST262_CHECKOUT" status --porcelain --untracked-files=all --ignored=matching)" &&
    test "$(cat "$TEST262_CHECKOUT/.git/refs/heads/main")" = "$test262_revision" &&
    GITHUB_SHA=337a3668a0dc86dd401ea20906e782249a64a228 \
    cargo run --release --offline --locked -j 1 -p boa_tester -- \
        run --test262-path "$TEST262_CHECKOUT" -o "$RESULTS_DIRECTORY"
```

Use a clean Test262 checkout with a loose local `main` ref at the pinned
revision, because the upstream JSON reporter reads that ref directly.
The [per-case reference report](test262-v0.22.json.gz) is the upstream
`latest.json`, compressed losslessly. Its uncompressed SHA-256 is
`d4cdacf5707861c8b59256e830e9807c4dd1bbc2478c4e4dfc17a76f91e3872e`.

The upstream runner reported 51,225 passing, 1,781 ignored, 572 failing
and zero panics across 53,578 test files. Passing cases account for
95.61% of the whole suite, including ignored cases in the denominator.
Where a file has both strict and non-strict modes, its passing outcome
requires both modes to pass. [The pinned configuration](../test262_config.toml)
contains the upstream exclusions.

Use the maintained runner with the same inputs and features to produce
a candidate JSON report. Compare case inventory and prior passes with
[compare_baseline.py](../tools/surgeist-js-test262/compare_baseline.py).
For the mechanical import, pass `--exact` to require identical outcomes.

## Performance

The original `boa_engine/full` binary uses the system allocator on this
host. It measures realm creation, parsing and execution over the built-in
fixtures. Its upstream “Compiler” cases reuse a script whose code block is
cached after the first iteration. Compare those cases as cached lookup
measurements.

The staged `surgeist-js-benchmarks/scripts` suite runs the same upstream
scripts and host registration with the system allocator. This deliberately
differs from upstream macOS mimalloc. Scripts run in stable path order
and verify their execution result. The 34 successful scripts are timed as
execution. `intl/numberformat-different-options` stops at Boa's unsupported
currency style and is timed as its expected `TypeError` path.
The V8 workloads catch their internal errors and accumulate an error count.
The harness requires that count to remain zero across warmup and measured
calls. It reads the count after each case, outside the timer.
For a split run, retain the 27 non-V8 cases separately from the eight
guarded V8 cases. The guards leave the non-V8 timed operations and case
names unchanged on macOS.
The staged `surgeist-js/compile`
benchmark measures bytecode compilation using fresh parsed scripts.
Parsing is setup for each iteration, outside the timed operation.

Use the same allocator, flags, suites and sample settings for the
post-conversion comparison. Run one benchmark executable at a time.
Use the upstream package for `full` and the staging packages for
`scripts` and `compile`.

```sh
cargo bench --offline --locked -j 1 -p boa_engine --bench full -- \
    --noplot --sample-size 30 --warm-up-time 1.5 --measurement-time 3
cargo bench --offline --locked -j 1 -p surgeist-js-benchmarks --bench scripts -- \
    --noplot --sample-size 30 --warm-up-time 1.5 --measurement-time 3
cargo bench --offline --locked -j 1 -p surgeist-js --bench compile -- \
    --noplot --sample-size 30 --warm-up-time 1.5 --measurement-time 3
```

The V8 script groups inherit a 10-sample setting and five-second measurement
target from Boa's Criterion harness. A slow case can exceed that target
when a single iteration per sample requires more time.
