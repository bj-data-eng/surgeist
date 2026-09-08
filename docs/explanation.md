# Explanation

## One Repository, Distinct Crate Boundaries

Surgeist keeps reusable UI contracts in separate crates while developing their
implementations in one Git repository and product Cargo workspace. Root owns
the facade, domain implementations, shared tooling, and integration. A change
to several participating crates can therefore share a source revision, review,
and dependency resolution without publishing intermediate crate candidates.

Domain ownership still matters. Root owns Surgeist-to-Surgeist adapters:
conversions from one crate's public model into another crate's public inputs.
Domain crates own their algorithms, models, and backend-local adapters, such as
text shaping, rendering backends, and platform hosting. `surgeist-test` owns
shared verification contracts. `surgeist-generator` supplies CSS corpus tooling
and browser-corpus mechanics, while layout keeps its semantic conversion and
adapter. Neither support crate is a facade production dependency.

The [repository guide](../AGENTS.md#product-boundary) defines these boundaries.
Models remain typed and composable, production dependencies remain directional
and acyclic, and symbolic values remain unresolved until their owning layer has
the necessary context. A shared repository does not resolve mismatched models,
feature incompatibilities, or missing integration behavior by itself.

## Snapshot Import Basis

The consolidation imports the 14 crate revisions selected by root commit
[`e0303b14ccd6a81cf3d092105201daf7797ef6aa`](https://github.com/bj-data-eng/surgeist/tree/e0303b14ccd6a81cf3d092105201daf7797ef6aa/crates)
and the separate `surgeist-generator` source at
[`17f6159a4adb18f0d03cab58f81ad635660e2054`](https://github.com/bj-data-eng/surgeist-generator/tree/17f6159a4adb18f0d03cab58f81ad635660e2054).
The imported files become ordinary root-owned source; the original crate Git
histories are not merged. Their upstream URLs and immutable source citations
remain useful provenance.

Every imported crate's `plans` directories are omitted. Source, tests, fixtures,
and legal material remain with their owning crates, subject to the reviewed
workspace and tooling adaptations. Root's existing `plans/` is retained. Its
completed records describe their original source basis and do not override
current manifests, ownership, or verification policy.

## What Remains From The Reset Baseline

The [August 9, 2026 reset specification](../plans/specs/2026-08-09-root-baseline-reset.md)
records the earlier transition to a small root facade with independent crates.
It retained production path dependencies, compatible feature forwards, direct
reexports, and root-owned API audits. It removed former adapters, integration
tests, requirements, examples, the native development harness, and a fixture
metadata tool.

That reset removed `surgeist::adapters` and the incompatible root `text-render`
feature without a compatibility shim. The `app` and `runtime` module names both
continue to expose the runtime crate. Source consolidation does not restore
those removed surfaces or supply an integrated UI application. Their future
implementation needs explicit product contracts and composition tests.

## Workspace Membership And Verification Are Separate

All 16 product packages share one committed root lockfile. The facade is the
sole default member, keeping unqualified root commands focused. Explicit package
selection exposes the rest of the workspace without automatically running its
CPU-heavy layout suites, GPU tests, platform hosts, or browser tooling.

Checks run serially with one Cargo job and one test thread, selected for the
affected domain and supported feature/platform configuration. This makes the
verification scope deliberate; it does not turn facade success into evidence
for every crate. The earlier reset's
[completed cycle](../plans/cycles/2026-08-09-root-baseline-reset-C01-latest-leaves.md)
is historical evidence, not a current whole-workspace test result.

The API generator stays in a separate Cargo workspace because its rustdoc
toolchain and dependencies serve source auditing. The optional layout Dylint
catalog also stays separate because it uses compiler-internal nightly tooling
for explicitly selected audit questions. Neither expands ordinary product
workspace verification.

## Why API Audits Stay Central

One API generator owns the facade and crate audit set. It discovers source
directories directly and includes support crates as audit inputs. Generated
text makes source changes inspectable, but it is neither the source of truth
nor a complete behavioral or compatibility test. Source and corresponding
generated changes can now be reviewed in the same repository change.

Use the [reference](reference.md) to locate current interfaces and the
[how-to guide](how-to.md) for maintenance procedures.
