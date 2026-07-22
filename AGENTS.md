# Surgeist Root Repository Guide

Use the installed `surgeist-agent` plugin for every task in this repository.
Select the task-appropriate focused skill.

## Authority Split

This file is the root repository's committed discovery entry point. It owns the
mapping from facts to authoritative sources, the intended ownership and
architecture boundaries, and the configured root command inventory. The sources
named below own their current values.

The installed `surgeist-agent` plugin is the sole Surgeist workflow authority.
Its selected skill owns scope control, planning, debugging and TDD,
worker/reviewer gates, external-software permission,
the absolute unsafe prohibition, Git landing and publication, handoffs, and
submodule promotion. This file does not redefine those procedures or grant
authority to mutate, install, commit, or publish.

Resolve an apparent conflict by domain: use this file and the sources below for
mutable repository facts; use the selected plugin skill for workflow.
Higher-priority user and system instructions still apply. Do not import another
workflow.

## Repository Identity And Ownership

This repository has two roles:

- the root `surgeist` facade crate in `src/`;
- the integration workspace that pins independent `surgeist-*` repositories as
  Git submodules under `crates/`.

Root owns the facade and public composition surface, Surgeist-to-Surgeist
adapters, workspace wiring, gitlinks, cross-crate plans, root integration tests
and tools, source-derived API artifacts, and whole-workspace verification.

Each leaf repository owns its domain implementation, manifest, public front
door, focused tests, docs, commits, and published candidate. A parent workspace,
Codex project, task, branch, or worktree does not change repository ownership.
Root may inspect leaf source for integration but does not implement leaf code.

## Discover The Current Structure

Read these sources in order. Do not substitute a cached roster or a descriptive
README list for them. In this file, `<crate>` means the exact leaf package name
from a `crates/<crate>` workspace member in root `Cargo.toml`.

| Fact | Authoritative source |
| --- | --- |
| Root package, MSRV, facade dependencies, features, workspace members | root `Cargo.toml` |
| Leaf repository paths and authoritative URLs | `.gitmodules` |
| Compatible leaf revisions currently selected by root | committed gitlinks; inspect with `git submodule status --recursive` |
| Leaf package identity, actual dependencies, and features | pinned `crates/<crate>/Cargo.toml` |
| Leaf role, front door, and crate-specific commands | pinned leaf `AGENTS.md`, `src/lib.rs`, README/task runner, and CI |
| Root public facade | `src/lib.rs` and its reexports |
| Root API generator and generated audit artifacts | `api/generator/`, `api/public-api.txt`, and `api/crates/` |
| Root-owned integration helpers | `dev/` and `tools/` manifests and source |

When these sources disagree, report the exact paths and pinned revisions. Do not
guess, silently update another document, or widen the task to reconcile them.

## Crate Roles

These are the intended ownership boundaries; manifests and source at the pinned
revisions provide the current implementation facts.

| Crate | Owns |
| --- | --- |
| root `surgeist` | Thin facade, public composition, cross-crate adapters, and integration |
| `surgeist-animation` | CSS animation and transition timing, easing, keyframes, interpolation, and sampled values |
| `surgeist-css` | Strict CSS syntax parsing and authored CSS values |
| `surgeist-dialog` | Dialog contracts and coordination primitives |
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

Add or repurpose a crate only for a durable API and ownership boundary. Update
this table in the same authorized architecture change.

## Architecture Boundaries

- Keep production dependencies directional and acyclic. The pinned manifests are
  the actual graph; a proposed new edge must preserve the roles above.
- Root composes leaf front-door APIs. Surgeist-to-Surgeist lowering belongs in
  root or a root-owned tool.
- Leaf internals are private by default. Do not reach through sibling private
  modules or duplicate cross-crate interpretation.
- A leaf owns backend-local adapters only when that backend belongs to its domain.
- `surgeist-test` may depend on production leaves for shared verification, but the
  root facade must not production-depend on `surgeist-test`.
- A small change requiring many leaf edits is evidence to revisit the boundary,
  not permission to create a dependency sink.

Surgeist is a reusable, host-adapter-agnostic Rust UI framework built from strict,
typed, composable primitives. Public APIs, internal models, errors, defaults,
features, tests, docs, and examples are all product contracts. Expose intentional
front doors and keep symbolic values unresolved until their owning layer has the
required context.

## API Artifacts

Source is authoritative; files under `api/` are generated audit artifacts. The
root owns the only API generator and all generated API artifacts. Leaf repositories
must not carry copies.

Configured root commands are:

```sh
cargo run --manifest-path api/generator/Cargo.toml
cargo run --manifest-path api/generator/Cargo.toml -- --check
cargo run --manifest-path api/generator/Cargo.toml -- --crate <crate>
```

Refresh artifacts only after the owning leaf source is committed, published, and
visible at the root gitlink. Commit generated deltas in root with the integration
change that exposed their source. Never hand-edit generated artifacts.

## Root Command Inventory

These commands describe root verification capability; the selected plugin
skill decides their exact gate, order, feature matrix, and whether
already-present tooling can run without unauthorized acquisition.

```sh
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
cargo run --manifest-path api/generator/Cargo.toml -- --check
```

Derive package-specific and feature-specific checks from the pinned owning
manifest and policy. Discovery is complete when the owning repository, pinned
revision, public entry point, dependency and feature facts, generated-artifact
owner, and applicable command inventory are identified without inventing policy.
