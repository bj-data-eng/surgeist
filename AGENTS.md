# Surgeist Repository Guide

Use `$pisct:coordination` for delivery coordination and a task-appropriate
focused PISCT skill for bounded work. Preserve a workflow explicitly selected
by higher-priority instructions. This guide grants no mutation, installation,
commit, publication, or cross-repository authority.

Use the repository-local
[$surgeist-admin](.agents/skills/surgeist-admin/SKILL.md) for Project coordination,
local working material, and pause/resume handoffs.
It owns Surgeist's administrative storage conventions; PISCT continues to own
the engineering workflow.

## Authority Split

This file is the repository's committed discovery entry point. It owns the
mapping from mutable repository facts to their authoritative sources, product
and ownership boundaries, and configured command inventory. Crate-local
`AGENTS.md` files supplement this guide with domain facts and focused checks;
they do not establish separate repository ownership or delivery workflows.

Apply `$pisct:invariants` to engineering work and `$pisct:rust-modeling` to Rust
model/API decisions. Higher-priority user and system instructions still apply.

The coordinator owns issue and Project updates, staging, commits, integration,
and publication within the user's authorized scope. Use `$pisct:holistic-review`
for final independent integration review.

## Repository Identity And Ownership

This repository owns the `surgeist` facade in `src/`, all crate implementations
under `crates/`, their manifests, public contracts, tests, fixtures, documentation,
and shared tooling. Repository root owns workspace wiring, cross-crate plans,
integration verification, and source-derived API artifacts. Each consuming
subsystem owns its integration with the lower-level crates it depends on.

Crates remain domain boundaries within one Git repository. Their original
repositories supplied the imported snapshots; they are historical source
origins, not a publication or promotion prerequisite for current development.
The [architecture explanation](docs/explanation.md) records the import basis and
the distinction between source consolidation and product integration.

## Discover The Current Structure

Read the sources below in order. Derive package identity and membership from
manifests, and implementation behavior from current source.

| Fact | Authoritative source |
| --- | --- |
| Root package, MSRV, facade dependencies, features, workspace members, and default members | Root `Cargo.toml` |
| Product dependency resolution | Root `Cargo.lock` |
| Crate identity, dependencies, features, and compatibility | Each `crates/surgeist-*/Cargo.toml` |
| Domain front doors and focused checks | Each crate's `AGENTS.md`, `src/lib.rs`, README, tests, and tracked scripts |
| Public facade and root package tests | `src/lib.rs` |
| API generator, target discovery, rendering, and tests | `api/generator/Cargo.toml` and `api/generator/src/` |
| Offline CSSOM property reflection generation | `tools/cssom-reflections/`; checked-in CSSOM reflection source |
| Shared CSS and browser-corpus generation | `crates/surgeist-generator/Cargo.toml`, `src/`, and documentation |
| Optional layout Dylint audit catalog | `crates/surgeist-layout/tools/surgeist-layout-audits/` |
| Generated API audits | `api/public-api.txt` and `api/crates/`; source remains authoritative |
| Human-facing overview and documentation portal | `README.md` and `docs/` |
| License, source attribution, and bundled-material provenance | `LICENSE`, `NOTICE.md`, `licenses/`, and crate-local notices and legal material |
| Verification commands | This guide's Command Inventory and the affected crate supplements |

`default-members = ["."]` keeps unqualified root Cargo commands focused on the
facade. One root `Cargo.lock` resolves the product workspace. The API generator
and nested layout Dylint catalog remain separate Cargo workspaces.

When sources disagree, report exact paths and revisions. Do not guess, silently
rewrite another authority, or widen the task to reconcile them.

## Product Boundary

Surgeist is a reusable, host-adapter-agnostic Rust UI framework built from strict,
typed, composable primitives.

The following are domain boundaries. Current manifests and source establish the
implemented dependency graph and behavior.

| Crate | Owns |
| --- | --- |
| Root `surgeist` | Thin facade, curated reexports, application builders, and bootstrap |
| `surgeist-animation` | CSS animation and transition timing, easing, keyframes, interpolation, and sampled values |
| `surgeist-color` | Dependency-free checked numerical colors, conversion, interpolation, transformations, resolved serialization, finite formatting, and byte-alpha spelling |
| `surgeist-css` | CSS syntax, authored values, browser recovery diagnostics, clean-report validation, and source/fact-qualified color resolution |
| `surgeist-cssom` | Live authored CSS identity and state, immutable snapshots, revisions, atomic edits, and its CSS integration adapter |
| `surgeist-dialog` | Dialog contracts and coordination primitives |
| `surgeist-dom` | Canonical tree identity, atomic edits, structural transition state, revisions and immutable snapshots |
| `surgeist-fuzz` | Bounded fuzz engines, generation support, process execution, evidence, replay, reduction and campaign tooling; consumers own cases and expectations |
| `surgeist-generator` | Shared generation contracts, CSS corpus driver, browser-corpus infrastructure, provenance, and publication of generated artifacts |
| `surgeist-html` | Pinned HTML values, local rules, native authoring and immutable construction |
| `surgeist-html-parser` | Decoded-Unicode ingestion, recovery, source provenance, construction effects and immutable parse reports |
| `surgeist-html-semantics` | Qualified captures, construction-to-capture import, external facts and bounded intrinsic semantic queries |
| `surgeist-layout` | Layout algorithms and contracts, layout-ready fixtures, and parity/oracle tests |
| `surgeist-render` | Rendering contracts and backend-facing draw data |
| `surgeist-retained` | Retained identity and state, tree identity, and stable handles |
| `surgeist-runtime` | App orchestration, events/effects, lifecycle, invalidation, frame scheduling, and provenance |
| `surgeist-shape` | Shape, geometry, and primitive path data |
| `surgeist-task` | Task scheduling, cancellation, progress, admission, and executor-facing policy |
| `surgeist-template` | Typed template and future DSL-facing authoring contracts |
| `surgeist-test` | Shared test schemas, harnesses, fixtures, and integration verification support |
| `surgeist-text` | Text shaping, measurement, font abstractions, and text layout |
| `surgeist-window` | Window, app-host, event-loop, and platform-host contracts |

- Update this table with authorized ownership changes.
- Follow the [architecture vision](https://chatgpt.com/space/page_6ce383176ca08191b1dd6d5d16e63979):
  subsystems depend directly on the lower-level contracts they consume and own
  their integration and focused tests. Keep dependencies acyclic and each
  semantic phase authoritative in one owner. Lower layers never depend on the
  facade or runtime; runtime owns orchestration and coherent publication.
  The facade is not the mandatory path for cross-crate interactions.
- `surgeist-test` may depend on production crates for shared verification. The
  facade must not production-depend on it or on corpus-generation tooling.
- `surgeist-fuzz` is shared verification tooling outside default members and
  facade dependencies. Crates own their fuzz packages, codecs, semantic
  properties, seeds and regressions. The framework has no crate-under-test
  dependency; its controlled author fixture is a separate test workspace.
  Benchmarking is outside its current scope.
- `surgeist-generator` owns shared CSS and browser-corpus mechanics; layout owns
  its semantic conversion and corpus adapter. Shared tooling does not absorb
  layout algorithms or application semantics.

The separate original source checkouts are outside this repository's
implementation scope.

The previous `surgeist-style` implementation is removed. `surgeist-cssom`
provides stable identities, immutable snapshots, atomic edits and headless live
declaration, rule, stylesheet and MediaList operations. Its crate README records
current extension operations and source-qualified capability limits.

`surgeist-html` owns exact shared values, local language rules, native builders
and the controlled immutable construction owner. `surgeist-html-parser` consumes
that owner and semantic captures for decoded-Unicode ordinary documents and
qualified fragments, with recovery, source provenance, bounded effects and
explicit terminal cleanup. `surgeist-html-semantics` owns captures and bounded
queries and adapts native or parsed HTML results through one public capture
adapter. Each crate's README records capability limits; these do not certify
full browser conformance. HTML has no parser, semantic, DOM, CSS or Color
dependency. Semantics depends only on HTML. Parser owns direct CSS grammar and
Color numerical integration through its original controller.
The HTML standard revision defines behavior; the html5ever revision identifies
implementation provenance. Ingestion may change implementation machinery but
must not introduce a Surgeist HTML dialect.

`surgeist-dom` begins the canonical live tree owner with native XML Documents,
namespaced Elements, ordinary DocumentFragment, Text, Comment,
ProcessingInstruction and DocumentType nodes. It provides ordered namespaced
attributes, atomic edits, same-store adoption, immutable-capture import with fresh
target identities, finite caller limits and owned snapshots. Native namespace
creation consumes HTML-owned initialization requirements and common values,
retaining applicable nonce state independently from attributes. The native
document environment is established independently of its XML class. Its crate
guide records this bounded surface. HTML document creation, remaining special
family state/hooks, complete HTML installation and shadow trees remain
contribution work;
the existing retained facade is not replaced by this foundation.
DOM depends directly on HTML and consumes its exact `DocumentString`. Its direct
HTML-semantics adapter builds qualified ordinary-tree captures, retains checked
bidirectional identity maps, and invokes the producer queries with explicit
availability and finite capture limits.
DOM's immutable external focus/hover captures use exact canonical document
stamps and a separate host source/revision. Runtime directly consumes DOM and
owns the adapter from its actual surface selections and checked associations.
DOM also consumes CSS's checked transition names and CSSOM's ordinary sheet/rule
identities for structural view-transition state. Runtime owns rendering resources
and composed readiness; DOM's structural records do not establish either.
Its remaining installation work owns importing completed HTML construction
values with their actual state and associations. `surgeist-html` owns
typed HTML construction; `surgeist-template` retains template syntax, symbolic
expressions, component validation and Rust generation, consuming the HTML and
DOM contracts directly rather than duplicating their semantics. HTML parse results
are immutable construction values, not a second live DOM. DOM retains generic
name/string/tree invariants; HTML owns its language-specific interpretation.
The remaining downstream owners are planned: `surgeist-selectors` for matching
and specificity, `surgeist-cascade` for precedence and rollback, and
`surgeist-style` for rule collection, inheritance, substitution, computed values
and invalidation. HTML-owned work precedes DOM, then the downstream CSS crates,
as directed by the user. DOM integration follows availability of both producers;
do not encode that scheduling choice as blanket blocking dependencies. Manifests
and source establish the implemented workspace and current capabilities.

## Backward Compatibility

Until the root `surgeist` public API reaches v1.x.x, do not add or retain
compatibility or migration shims or machinery in any Surgeist crate. Reaching
that version permits consideration of such support; it does not authorize it
automatically. CSS-standard legacy syntax and aliases express CSS semantics
and are not crate compatibility shims. Apply `$pisct:invariants` for semantic
API naming.

## Subagent Models

Use GPT-6.1 Sol (`gpt-6.1-sol`) for every subagent, with no automatic alternatives.
The coordinator also uses GPT-6.1 Sol. Follow `$pisct:coordination` for explicit
launch settings, assignment-specific reasoning effort, and review independence.

## Generated Artifacts

The API generator lives in `api/generator/`; its outputs are
`api/public-api.txt` and `api/crates/*.txt`. It scans `crates/` for `surgeist-*`
directories containing manifests, independently of Cargo workspace membership.
This includes the shared generator crate and test-support crate. Inspect
`api/generator/src/lib.rs` for target selection and its pinned rustdoc toolchain.

The [API audit reference](docs/reference.md#source-and-artifact-locations)
describes default-feature and CSS corpus profile artifacts. Use the
[audit procedures](docs/how-to.md#check-public-api-audits) for checks and
authorized refreshes. No external crate publication or pointer update is required.
Keep the generator in its root owner; do not copy it into crates.

The separate root-owned [CSSOM reflection tool](tools/cssom-reflections/README.md)
generates checked-in ordinary property accessors from public CSS metadata.
It is a workspace tool outside default members and production dependencies.
Its check mode verifies reproducibility and the actual audited structural-method
inventories; API freshness remains the separate auditor's responsibility.

Corpus generators and inputs remain distinct from API auditing. Use
`$pisct:invariants` for generated-artifact handling and acquisition authority,
and `$pisct:attribution` for bundled-material provenance.

## Command Inventory

Targeted Cargo check, build, or test commands may run in parallel, with one
Cargo build job per command. Monitor `pisct host status` and follow
`$pisct:coordination` for resource-pressure handling. Use separate Cargo target
directories for concurrent worktrees changing the same packages; reuse the
existing target when the source and verification context permit. Use default
test-harness parallelism unless host status advises a limit.
Do not use blanket workspace test runs. Select focused verification through
`$pisct:testing` and the affected crate's supplement; execute commands through
`$pisct:process`.

The small facade checks from the repository root are:

```sh
cargo check --offline --locked -j 1 -p surgeist
cargo test --offline --locked -j 1 -p surgeist --lib
cargo clippy --offline --locked -j 1 -p surgeist --all-targets -- -F unsafe-code -D warnings
cargo fmt --all -- --check
```

The optional layout Dylint catalog is separately selected and is not a product
test, default workspace member, or standing verification gate.

API-generator tests and audit commands use its separate workspace. The
environment setting also limits nested rustdoc Cargo builds to one job:

```sh
CARGO_BUILD_JOBS=1 cargo test --offline -j 1 --manifest-path api/generator/Cargo.toml
CARGO_BUILD_JOBS=1 CARGO_NET_OFFLINE=true cargo run --offline -j 1 --manifest-path api/generator/Cargo.toml -- --list
CARGO_BUILD_JOBS=1 CARGO_NET_OFFLINE=true cargo run --offline -j 1 --manifest-path api/generator/Cargo.toml -- --check
```

The API tool's real feature fixture is ignored in the ordinary suite because it
invokes the pinned nightly rustdoc for generation and checking. Select it
separately when profile handling changes. Unset `CARGO_TARGET_DIR` so the fixture
owns and removes its nested build output:

```sh
env -u CARGO_TARGET_DIR CARGO_BUILD_JOBS=1 CARGO_NET_OFFLINE=true cargo test --offline -j 1 --manifest-path api/generator/Cargo.toml --lib tests::css_corpus_audit_uses_only_requested_features_and_reports_its_missing_artifact -- --ignored --exact
```

API auditing is explicit maintenance and is not part of normal product tests.
