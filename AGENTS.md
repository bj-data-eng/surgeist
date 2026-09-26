# Surgeist Repository Guide

Use `$pisct:coordination` for delivery coordination and a task-appropriate
focused PISCT skill for bounded work. Preserve a workflow explicitly selected
by higher-priority instructions. This guide grants no mutation, installation,
commit, publication, or cross-repository authority.

Use the repository-local
[$surgeist-admin](.agents/skills/surgeist-admin/SKILL.md) for plans, ledger
organization, evidence retention, temporary files, and pause/resume handoffs.
It owns Surgeist's administrative storage conventions; PISCT continues to own
the engineering workflow.

## Authority Split

This file is the repository's committed discovery entry point. It owns the
mapping from mutable repository facts to their authoritative sources, product
and ownership boundaries, and configured command inventory. Crate-local
`AGENTS.md` files supplement this guide with domain facts and focused checks;
they do not establish separate repository ownership or delivery workflows.

PISCT skills supply reusable coordination and engineering guidance. Apply
`$pisct:invariants` to engineering work, including its absolute unsafe prohibition
to all Surgeist-owned Rust. There is no exception for executable `unsafe` in
Surgeist-owned code. Higher-priority user and system instructions still apply.

## Repository Identity And Ownership

This repository owns the `surgeist` facade in `src/`, all crate implementations
under `crates/`, their manifests, public contracts, tests, fixtures, documentation,
and shared tooling. Root also owns Surgeist-to-Surgeist adapters, workspace
wiring, cross-crate plans, integration verification, and source-derived API
artifacts. Changes spanning these areas can be developed and reviewed together
within the task's authorized scope.

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

The product Cargo workspace contains root plus 15 crates: 16 members in total.
`default-members = ["."]` keeps unqualified root Cargo commands focused on the
facade. Its 13 production path dependencies compile as needed; membership does
not make `surgeist-test` or `surgeist-generator` facade dependencies or reexports.
One committed root `Cargo.lock` resolves the product workspace. The API generator
and nested layout Dylint catalog remain separate Cargo workspaces.

When sources disagree, report exact paths and revisions. Do not guess, silently
rewrite another authority, or widen the task to reconcile them.

## Product Boundary

Surgeist is a reusable, host-adapter-agnostic Rust UI framework built from strict,
typed, composable primitives. Public APIs, internal models, errors, defaults,
features, tests, docs, and examples are product contracts. Keep symbolic values
unresolved until their owning layer has the required context.

The following are domain boundaries. Current manifests and source establish the
implemented dependency graph and behavior.

| Crate | Owns |
| --- | --- |
| Root `surgeist` | Thin facade, public composition, cross-crate adapters, and integration |
| `surgeist-animation` | CSS animation and transition timing, easing, keyframes, interpolation, and sampled values |
| `surgeist-css` | CSS syntax, authored values, browser recovery diagnostics, and clean-report validation |
| `surgeist-dialog` | Dialog contracts and coordination primitives |
| `surgeist-generator` | Shared generation contracts, CSS corpus driver, browser-corpus infrastructure, provenance, and publication of generated artifacts |
| `surgeist-layout` | Layout algorithms and contracts, layout-ready fixtures, and parity/oracle tests |
| `surgeist-render` | Rendering contracts and backend-facing draw data |
| `surgeist-retained` | Retained identity and state, tree identity, and stable handles |
| `surgeist-runtime` | App orchestration, events/effects, lifecycle, invalidation, frame scheduling, and provenance |
| `surgeist-shape` | Shape, geometry, and primitive path data |
| `surgeist-style` | Style model, cascade, resolution, validation, and invalidation |
| `surgeist-task` | Task scheduling, cancellation, progress, admission, and executor-facing policy |
| `surgeist-template` | Typed template and future DSL-facing authoring contracts |
| `surgeist-test` | Shared test schemas, harnesses, fixtures, and integration verification support |
| `surgeist-text` | Text shaping, measurement, font abstractions, and text layout |
| `surgeist-window` | Window, app-host, event-loop, and platform-host contracts |

- Add or repurpose a crate only for a durable API and domain boundary. Update
  this table in the same authorized architecture change.
- Keep production dependencies directional and acyclic. A dependency edge must
  preserve the roles above.
- Compose intentional front-door APIs. Surgeist-to-Surgeist lowering belongs in
  root or a root-owned tool; backend-local adapters remain in their domain crate.
- Keep crate internals private. Do not reach through sibling private modules or
  duplicate cross-crate interpretation.
- `surgeist-test` may depend on production crates for shared verification. The
  facade must not production-depend on it or on corpus-generation tooling.
- `surgeist-generator` owns shared CSS and browser-corpus mechanics; layout owns
  its semantic conversion and corpus adapter. Shared tooling does not absorb
  layout algorithms or application semantics.
- A small change requiring many crate edits is evidence to revisit the boundary,
  not permission to create a dependency sink.

Inspecting another repository never grants write authority there. The separate
original source checkouts are outside this repository's implementation scope.

## Generated Artifacts

The API generator lives in `api/generator/`; its outputs are
`api/public-api.txt` and `api/crates/*.txt`. It scans `crates/` for `surgeist-*`
directories containing manifests, independently of Cargo workspace membership.
This includes the shared generator crate and test-support crate. Inspect
`api/generator/src/lib.rs` for target selection and its pinned rustdoc toolchain.

Each package retains its default-feature artifact. The configured
`surgeist-generator` CSS profile additionally writes
`api/crates/surgeist-generator.css-corpus.txt` using `--no-default-features
--features css-corpus`; it does not enable `browser-corpus`. Both `--crate
surgeist-generator` and `--all` select its default and CSS artifacts. Listings,
artifact headers, and stale-artifact diagnostics identify the CSS profile.

Refresh API audits from the authorized source change in this repository and
review the generated diff with that change. No external crate publication or
pointer update is required. Source is authoritative; never hand-edit generated
artifacts or carry additional API generator copies inside crates.

Corpus generators and their inputs remain distinct from API auditing. Keep
bundled fixture provenance, license material, and generated expectations with
their owning corpus. Run mutation, browser acquisition, or source acquisition
only within the task's explicit authorization; a verification command inventory
does not grant that authority.

## Command Inventory

Run one top-level Cargo check, build, or test command at a time, with one Cargo
build job.
Allow the test harness to use its default parallel test threads. Limit test
threads only when `pisct host status` reports high resource pressure. Record
the host reading and selected thread limit with the check result, and restore
the default parallel harness when that pressure clears.
Select the affected package, target, feature combination, and test suite before
execution. Do not use blanket workspace test runs or assume `--all-features` is
supported. Workspace membership alone does not select a verification matrix.
Use `$pisct:process` for authorized noninteractive checks with already-present
tooling; missing prerequisites require setup authority, not an online retry.

The small facade checks from the repository root are:

```sh
cargo check --offline --locked -j 1 -p surgeist
cargo test --offline --locked -j 1 -p surgeist --lib
cargo clippy --offline --locked -j 1 -p surgeist --all-targets -- -F unsafe-code -D warnings
cargo fmt --all -- --check
```

For an affected crate, use its supplement to select focused checks and keep the
same Cargo execution limits and default parallel test harness. For example,
task-library tests are:

```sh
cargo test --offline --locked -j 1 -p surgeist-task --lib
```

Native GPU, window, browser, corpus, platform, and executor checks have distinct
prerequisites and resource costs. Run only the applicable documented checks;
neither facade success nor one feature combination proves those suites pass.
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

For an authorized refresh, select `--root`, `--crate surgeist-task`, or `--all`
after `--` in the same generator command. API auditing is explicit maintenance
and is not part of normal product tests. A successful check reports current
selected artifacts; a stale artifact returns to source-led regeneration and
diff review. Stop on missing prerequisites or an unexplained generated delta.

Discovery is complete when ownership, product boundaries, public entry points,
dependencies, feature and platform constraints, artifact ownership, and the
applicable verification commands and concurrency constraints are established
from current source.
