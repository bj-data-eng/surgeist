# Safe JavaScript engine performance baseline

The comparison pairs 128 reference cases and three supplemental buffer
cases. [Per-case results](benchmark-results.csv) include the median,
95% confidence interval, and sample count for each build.
[Raw samples and run logs](criterion-samples.tar.gz) retain individual
Criterion estimates and host readings.

## Elapsed time per iteration

Each row shows the median of its per-case safe-to-reference time ratios.
A ratio above 1 means the safe build took longer per iteration.

| Workload | Paired cases | Safe / Boa v0.22 | Case range |
| --- | ---: | ---: | ---: |
| Realm creation | 1 | 4.85× | 4.85× |
| Parsing | 23 | 0.97× | 0.89–1.03× |
| Fresh bytecode compilation | 23 | 1.01× | 0.96–1.13× |
| Cached bytecode lookup | 23 | 12.43× | 2.06–12.53× |
| Built-in execution fixtures | 23 | 4.87× | 1.49–6.47× |
| Non-V8 scripts | 27 | 3.11× | 0.93–8.30× |
| V8 scripts | 8 | 2.22× | 1.47–4.07× |
| Supplemental buffers | 3 | 5.43× | 3.87–5.95× |

Realm creation moved from 72.5 to 351.6 µs. The Object Creation cached
lookup moved from 2.48 to 31.0 ns. The closure script moved from
0.975 to 8.09 s, and Earley–Boyer moved from 16.8 to 68.5 s.
The scripts suite measures the known unsupported Intl currency case
as an error path. Both V8 runs passed the internal error guards.

## Process memory

Peak resident set size measures the entire process for each cohort,
including the benchmark harness and all cases in that run.

| Comparable cohort | Boa v0.22 | Safe |
| --- | ---: | ---: |
| Full suite | 12.00 GB | 132.8 MB |
| Fresh compilation | 72.9 MB | 46.4 MB |
| Buffers | 27.6 MB | 26.8 MB |
| Final five V8 scripts | 533.6 MB | 1,066.8 MB |

The initial reference non-V8 and three-case V8 cohorts were interrupted
after their retained samples finished. Those logs lack process peak RSS.

## Measurement basis

The reference is Boa v0.22 at
[`337a3668`](https://github.com/boa-dev/boa/tree/337a3668a0dc86dd401ea20906e782249a64a228).
The initial 100 saved cases and the remaining cases used the pristine
engine or the preserved pre-conversion reference executables. The safe
original workloads use workspace tree
`d7ce9efe497c91ecba82b25e3663dc1b1e0db14d`. The supplemental buffer
workload uses tree `cb1101a1be79633522e53ffacdc3ae9db9f708e7`.

Both builds used the system allocator on Apple M3 with Rust 1.99.1,
fat LTO, and one codegen unit. Each ordinary case used 30 samples,
1.5 seconds of warmup, and a three-second measurement target. V8 cases
used ten samples and a five-second target, with longer collection when
one iteration exceeded that target. [Build and run instructions](README.md#performance).

The results compare complete builds. The shared dependency versions differ
for 22 registry package names in the engine suites and 23 in the script and
buffer suites. The sample archive includes those graph differences, binary
and lockfile hashes, and host readings. Per-collection timing was unavailable
in the reference harness.
