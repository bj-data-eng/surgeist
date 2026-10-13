# surgeist-fuzz

`surgeist-fuzz` runs bounded verification and retains reproducible evidence.
Each consuming crate owns its cases and semantic expectations. This framework
supplies common generation helpers, engine adapters, process execution,
evidence storage, replay, reduction and campaign control. It does not decide
whether a CSS value, HTML tree, cascade result or layout result is correct.
Benchmarking is outside the current scope.

## Author a case once

Implement `CaseTarget` in the owner's fuzz package. `generate` interprets engine
entropy and materializes a concrete case; `encode` and `decode` define the
versioned canonical replay format; `evaluate` checks independently justified
properties. The same evaluator is used by property testing, coverage testing
and replay. `reduction_candidates` optionally supplies smaller domain-valid
cases. Every evaluation starts with fresh state.

For raw input, the codec can preserve bytes directly. Structured input needs
an explicit lossless codec. Arbitrary's byte stream is generation entropy; it
is not that codec. Preserve code units, ordering, stable fixture identities,
operation sequences and context that affect the result. Increment the codec
revision when its interpretation changes and the generator revision when
generation changes. Unsupported revisions are rejected, without migration
machinery.

The [controlled author fixture](tests/fixtures/subject/fuzz/) demonstrates this
workflow. Its toy predicates deliberately contain detectable failures. These
are framework smoke controls, not properties of any production crate.

### Register the owner package

Use an owner-local package such as `crates/surgeist-css/fuzz/`, named
`surgeist-css-fuzz`. Add it to the root workspace outside `default-members`.
It may depend on its production owner and on this framework; production code
must not depend on the fuzz package. Keep the product's single root lockfile.

Declare one descriptor file in the package manifest:

```toml
[package.metadata.surgeist-fuzz]
descriptor = "fuzz.toml"
```

The descriptor has schema `1`, a nonempty `targets` list and a `fixtures` list
of contained owner-local paths. Each target declares an owner-qualified ID,
binary, properties and requirement sources, codec and generator identities,
encoding, input bounds, build features/capabilities, minimum evaluated count
and generation-rejection allowance. See the executable fixture's
[descriptor](tests/fixtures/subject/fuzz/fuzz.toml) for the complete schema.
Discovery validates package ownership, binary/feature names and fixture paths.
It uses Cargo metadata and the declared file, with no central target list.

An owner binary normally calls `worker::serve(&target)`. The framed protocol
captures canonical bytes before evaluation, then returns the observation.
With its explicit coverage feature, the same named binary uses libfuzzer-sys's
safe `fuzz_target!` macro and `CoverageDriver::libfuzzer_entry_framed`. The
ordinary build remains a replay worker; the separately instrumented build is
the coverage executable. Keep libfuzzer-sys optional in the owner package.
The shared framework does not link libFuzzer into ordinary builds.
Construct campaign drivers with `CoverageDriver::for_campaign`: the command
passes a positive attempt limit, and the adapter reserves one attempt before
generation or evaluation, including initial corpus inputs. Once exhausted,
subsequent callbacks produce no owner calls or observations. Unchecked product
observations and generation errors still consume an attempt; they do not become
successful evaluations merely to fill the budget.

### Define meaningful expectations

Properties name the requirement or independent oracle they verify. Expected
product rejection counts as an evaluation only when the owner identifies
actual checked properties; an unchecked rejection is an observation.
Generation discard is separate. No evaluated cases means no verification.
Generated runs must meet their declared minimum count and rejection allowance.

Use `generation::Budget` to admit byte, text, node, depth, edge and operation
costs before construction. Its flat `OrderedForest` checks references and
traverses iteratively. It describes fixture topology, not HTML/DOM semantics.
Generic payloads must supply their own resource charges and control their own
allocation, traversal and destruction. Budget accounting is not a claim to
bound allocator overhead or arbitrary product memory.

The `property` feature provides real proptest strategies and a bounded runner,
plus proptest-state-machine authoring support. The `coverage` feature supplies
Arbitrary helpers and the coverage adapter. Numeric helpers preserve exceptional
floating-point bit patterns and integer boundary values. UTF-16 helpers preserve
unpaired code units; UTF-8 helpers reject invalid input instead of rewriting it.

## Evidence and replay

Failures retain canonical input bytes separately from strict JSON metadata.
The metadata records the target/codec/generator, source revision, lockfile and
executable digests, toolchain, engine, sanitizer, platform, features, providers,
finite run settings and observation. A seed alone cannot reproduce a structured
case and is never substituted for its concrete bytes.

Unknown process termination after capture has a separate `CaptureArtifact`.
It retains the concrete input and process evidence without inventing a property
or failure key. Termination before capture reports unavailable evidence.
Diagnostic output and cleanup status remain explicit.

`ArtifactStore` operates on a caller-owned root with entry, byte, input and
metadata quotas. Handle-relative no-follow operations reject path escapes and
symlinks. Evidence bundles are immutable; the commit marker is published last.
Every published run records its terminal outcome separately from counts and
cleanup. `report` includes these validated records, preserving cancellation,
resource stops and insufficient evaluation even when their counts coincide.
An interrupted reduction retains an incomplete report and its previously
verified best input; a response cannot confirm a candidate after failed cleanup
or abnormal process termination.

Interrupted bundles are reported as partial recovery material and cannot be
loaded as complete evidence. A duplicate is accepted only if its bytes and
metadata match. Concurrent writers wait up to five seconds for the publication
lock; persistent contention reports a resource stop before writing. Do not use
generated corpus directories as crash archives.

Exact replay requires the recorded build identity. Comparing with a changed
build is an explicit operation that writes a separate comparison record and
preserves the original. Keep checked-in seeds and promoted regressions separate
from writable corpus, captures and findings. Promotion is a manual owner action:
review the case, add a focused expectation with its requirement source, then
commit it in the consuming crate.

## Reduction

Reduction reconfirms the original and each accepted result against the exact
property, class and stable discriminator. Different failures, expected rejection
and generation discard do not qualify. Repeated confirmations expose observed
flakiness. Attempt, time and output limits bound the search; cancellation retains
the best verified result. The report preserves the original, accepted chain and
replay observations. It does not promise a global minimum.

Raw inputs use bounded deletion and byte simplification. Structured cases use
owner candidates so identities, references and context stay valid. Canonical
bytes are verified separately when re-encoding changes the submitted bytes.
Campaigns use isolated replays through the common process executor;
`reduce_direct` is only an in-process convenience and cannot interrupt a hung
owner call. An unqualified panic/crash key cannot prove the same bug and is
unsupported for semantic reduction.

For a raw case captured from a coverage build, `minimize --engine-proposal`
first asks that recorded engine for a smaller candidate, then verifies it through
ordinary owner replay. This requires a dated nightly and a whole-second
`--case-ms`. It also verifies that passing the raw canonical bytes back through
owner generation yields exactly those bytes. Structured cases use owner
candidates instead. Engine proposal output is recorded separately from replay
observations and consumes the same total output and wall budgets. `--cases`
bounds attempts per engine process; the engine can launch child processes during
reduction. `--attempts` bounds ordinary confirmation replays, while the shared
wall deadline bounds the complete operation and descendant cleanup remains
separately bounded.

`minimize --corpus /absolute/prior/run/entropy-corpus` asks the recorded engine
to reduce coverage inputs. It copies the originals, writes a separate proposed
corpus and records the process outcome in `proposal.json`. A successful proposal
can be supplied to `run --restart-corpus`; failure artifacts and the original
corpus remain unchanged. The corpus and raw proposal modes are mutually
exclusive. Both use explicit storage bounds and the selected coarse engine
memory guard, without claiming an OS memory limit.

Tooling probes confirmed that `cargo-fuzz tmin` can switch to an unrelated
panic and `cmin` can remove crashing corpus seeds. Their exit status alone is
not evidence of same-failure reduction or preservation of failures.

## Execution and limits

The initial process backend supports Apple-Silicon macOS. It runs one owned
process group, transports bounded stdin/stdout/stderr without blocking helper
threads, observes cancellation and deadlines, escalates TERM to KILL when
needed, reaps the child and checks descendant group disappearance. Cleanup
failure remains distinct from the worker result. It never discovers unrelated
processes for termination.

Memory policy must be explicit. This backend supports accepted unbounded
memory and rejects a required hard limit before spawning. RSS sampling is
unavailable and reported as such. Input/structure admission, duration, output
and storage limits are enforced at their stated boundaries; they are not a
hard whole-process memory ceiling.

Coverage protocol output has a finite quota. A campaign stopped by that quota
is a resource stop, not a successful verification. Source coverage reports
additionally require compiler LLVM tools; sanitizer-guided feedback does not.

## Run a finite campaign

The command uses `clap` for subcommands, generated help, and argument errors.
Build it explicitly:

```sh
cargo build --offline --locked -j 1 -p surgeist-fuzz --features execution --bin surgeist-fuzz
target/debug/surgeist-fuzz --help
target/debug/surgeist-fuzz check --help
```

Prepare a clean, committed source checkout separately from active implementation.
The command does not create or update that checkout. Put campaign output outside
it. Toolchains and owner dependencies must already be available: discovery and
builds run offline, locked, and with one Cargo job. The framework never downloads
a tool or dependency during a campaign. Its locked Cargo build invokes the
libFuzzer target directly; `cargo-fuzz` is available for separate owner tooling
but is not a hidden build or installation step.

For an owner that has registered `surgeist-example::bytes`, use:

```sh
fuzz_source=/absolute/path/to/clean/source
fuzz_work=/absolute/path/to/owned/campaign
target/debug/surgeist-fuzz list --source "$fuzz_source" --crate surgeist-example
target/debug/surgeist-fuzz validate --source "$fuzz_source" --crate surgeist-example
target/debug/surgeist-fuzz check --source "$fuzz_source" --work "$fuzz_work" \
  --target surgeist-example::bytes --toolchain 1.97.0 \
  --memory-policy unbounded --cases 100 --wall-ms 30000 --case-ms 1000
target/debug/surgeist-fuzz run --source "$fuzz_source" --work "$fuzz_work" \
  --target surgeist-example::bytes --toolchain nightly-2026-09-01 \
  --memory-policy unbounded --cases 1000 --wall-ms 30000 --case-ms 1000
```

The example owner is a placeholder. The checked-in executable example uses
`surgeist-fixture::bytes` or `surgeist-fixture::structured` with
`--manifest crates/surgeist-fuzz/tests/fixtures/subject/Cargo.toml`.
Review the owner's requirement sources and independent expectations before
interpreting its successful result. `check --input /absolute/canonical.case`
replays the supplied case; `run --input /absolute/entropy.seed` supplies generation
entropy. Structured entropy is not a canonical case file.

`check --regressions /absolute/owner/fuzz/regressions` evaluates every selected
regular file once in deterministic path order. The directory must belong to a
registered fixture path; traversal and symlinks are rejected. Set `--cases` high
enough to admit all selected files. An empty directory reports `empty` without
falling back to generated cases. Checked-in regressions remain immutable.

Each command prints a JSON summary on stdout and a short human summary on stderr.
Exit codes are: `0` sufficient observed evaluations, `1` finding, `2` invalid or
unsupported request, `3` infrastructure failure, `4` cancellation, `5` resource
stop, `6` insufficient evaluations, and `7` empty selection. These distinguish
test results from operational failure; none establishes exhaustive correctness.

The default is one worker. `--workers N` explicitly partitions check/run budgets
across at most eight workers. Instrumented per-case deadlines are enforced in
whole seconds; `run` rejects a `--case-ms` value that is not a multiple of 1000.
The shared supervisor separately enforces the exact campaign wall deadline.
`--memory-policy unbounded` accepts this backend's stated memory capability;
required hard limits are rejected. The engine's RSS guard is coarse and is not
OS memory enforcement. No RSS sample is fabricated.

`--storage-bytes` and `--storage-entries` bound the immutable artifact store and
each engine corpus separately; build products are outside those evidence quotas.
Protocol output has a separate bounded allowance. A stopped or failed publication
cannot be reported as successfully retained evidence. Preserve originals when
making space; never replace a failure archive with a minimized coverage corpus.

The summary links an immutable artifact such as `failures/<identity>`. Reuse its
source, target, toolchain, and feature selection for exact replay and reduction:

```sh
target/debug/surgeist-fuzz replay --source "$fuzz_source" --work "$fuzz_work" \
  --target surgeist-example::bytes --toolchain 1.97.0 \
  --memory-policy unbounded --artifact failures/IDENTITY
target/debug/surgeist-fuzz minimize --source "$fuzz_source" --work "$fuzz_work" \
  --target surgeist-example::bytes --toolchain 1.97.0 \
  --memory-policy unbounded --artifact failures/IDENTITY \
  --attempts 100 --confirmations 2 --wall-ms 30000 --case-ms 1000
target/debug/surgeist-fuzz report --source "$fuzz_source" --work "$fuzz_work"
```

`replay --comparison` explicitly selects another clean source revision, records
the new observation, and preserves the original. `minimize` requires an exact
build and records both its actual limits and the original run configuration.
Process captures without semantic failure keys can be replayed, but cannot be
claimed as same-failure reductions.

SIGINT/SIGTERM cancel a running command. Alternatively, supply
`--cancel-file /absolute/path/to/not-yet-existing-file` and create that file when
you want cancellation. The command awaits owned process cleanup. A cleanup
failure remains visible even if a property finding was observed.
`run --restart-corpus /absolute/prior/run/entropy-corpus` starts a new recorded
run using a compatible prior corpus and its build metadata. It does not restore
the engine's in-memory state.

Promote a regression manually after inspecting its independent property and
replaying the exact case. `ArtifactStore` provides explicit export methods that
copy the case and metadata to a caller-owned store under the same quotas; they
do not include repository source or build caches. No command accepts regressions,
edits production source, files issues, or uploads artifacts automatically.

## Build and verify

The framework is a workspace member outside the facade's default dependency
graph. Build its command front door explicitly with `--features execution`.
Ordinary library checks do not require a C++ compiler or nightly compiler.
Coverage owners require a supported nightly, libfuzzer-sys and a C++ compiler.
The admitted tool combination is recorded in the fixture lockfile and source;
tool acquisition is a setup action, never a hidden campaign step.

```sh
cargo test --offline --locked -j 1 -p surgeist-fuzz
cargo test --offline --locked -j 1 -p surgeist-fuzz --all-features
cargo clippy --offline --locked -j 1 -p surgeist-fuzz --all-targets --all-features -- -F unsafe-code -D warnings
cargo test --offline --locked -j 1 -p surgeist-fuzz --features execution --test campaign -- --ignored
```

See [crate guidance](AGENTS.md) for focused checks and the [notice](NOTICE.md)
for this source distribution's direct dependency attribution.
