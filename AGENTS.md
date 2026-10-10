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
and shared tooling. Root also owns Surgeist-to-Surgeist adapters, workspace
wiring, cross-crate plans, integration verification, and source-derived API
artifacts.

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
| Root `surgeist` | Thin facade, public composition, cross-crate adapters, and integration |
| `surgeist-animation` | CSS animation and transition timing, easing, keyframes, interpolation, and sampled values |
| `surgeist-css` | CSS syntax, authored values, browser recovery diagnostics, and clean-report validation |
| `surgeist-cssom` | Live authored CSS identity and state, immutable snapshots, revisions, atomic edits, and its CSS integration adapter |
| `surgeist-dialog` | Dialog contracts and coordination primitives |
| `surgeist-generator` | Shared generation contracts, CSS corpus driver, browser-corpus infrastructure, provenance, and publication of generated artifacts |
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
- `surgeist-cssom` owns its integration adapter with `surgeist-css`. Other
  Surgeist-to-Surgeist lowering belongs in root or a root-owned tool;
  backend-local adapters remain in their domain crate.
- `surgeist-test` may depend on production crates for shared verification. The
  facade must not production-depend on it or on corpus-generation tooling.
- `surgeist-generator` owns shared CSS and browser-corpus mechanics; layout owns
  its semantic conversion and corpus adapter. Shared tooling does not absorb
  layout algorithms or application semantics.

The separate original source checkouts are outside this repository's
implementation scope.

The previous `surgeist-style` implementation is removed. `surgeist-cssom`
provides stable identities, immutable snapshots, atomic edits and headless live
declaration, rule, stylesheet and MediaList operations. Its crate README records
current extension operations and source-qualified capability limits. Greenfield
style follows the CSSOM contribution. Future style consumes CSSOM snapshots and owns
matching, cascade, inheritance, substitution, computed values, and invalidation;
CSSOM does not depend on style. Manifests and source establish the implemented
workspace and current capabilities.

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
