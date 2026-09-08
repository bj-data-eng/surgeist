# Surgeist Root Repository Guide

Use `$pisct:coordination` for standalone root delivery and a task-appropriate
focused PISCT skill for bounded work. Use `$pisct:plane-coordination` only
when plane coordination is explicitly selected. Preserve a workflow explicitly
selected by higher-priority instructions. This guide grants no mutation,
installation, commit, publication, or cross-repository authority.

## Authority Split

This file is the root repository's committed discovery entry point. It owns the
mapping from mutable repository facts to their authoritative sources, the
intended ownership and architecture boundaries, the configured root command
inventory, and the standalone PISCT workflow selection.

PISCT skills supply reusable coordination and engineering guidance. Apply
`$pisct:invariants` to engineering work, including its absolute unsafe prohibition
to all Surgeist-owned Rust. There is no exception for executable `unsafe` in
Surgeist-owned code. The selected skill supplies the discipline for the assigned
work; it does not change repository ownership or the caller's authorization.
Higher-priority user and system instructions still apply.

## Repository Identity And Ownership

This repository owns the `surgeist` facade crate in `src/` and the product
integration boundary that pins independent leaf repositories as Git submodules
under `crates/`. Root owns the public composition surface, Surgeist-to-Surgeist
adapters, workspace wiring, gitlinks, cross-crate plans, root integration tests
and tools, source-derived API artifacts, and product composition verification.
These responsibilities do not imply that every integration surface is currently
implemented; inspect the current source and README.

Each leaf repository owns its domain implementation, manifest, public front
door, focused tests, docs, commits, and published candidate. Placement in a parent
workspace, project, task, branch, or worktree does not transfer ownership. Root
may inspect leaf source for integration; root work does not authorize leaf edits.

## Discover The Current Structure

Read the sources below in order. Do not substitute a cached roster or a
descriptive README list for manifests, submodule configuration, or committed
gitlinks. A leaf path is an exact submodule path from `.gitmodules`; derive its
package name from the pinned leaf manifest, not from Cargo workspace membership.

| Fact | Authoritative source |
| --- | --- |
| Root package, MSRV, facade dependencies, features, and Cargo workspace membership | Root `Cargo.toml` |
| Leaf repository paths and authoritative URLs | `.gitmodules` |
| Leaf revisions selected by root and checked-out submodule state | Committed gitlinks (`git ls-tree HEAD crates/`) and `git submodule status --recursive` |
| Leaf package identity, dependencies, features, and compatibility | Each selected leaf's pinned `Cargo.toml` |
| Leaf role, public front door, and focused command inventory | Each selected leaf's pinned `AGENTS.md`, `src/lib.rs`, README, and any tracked task runner or CI |
| Root public facade and root package tests | `src/lib.rs` and its reexports |
| API generator package, target discovery, rendering, and generator tests | `api/generator/Cargo.toml` and `api/generator/src/` |
| Generated API audits | `api/public-api.txt` and `api/crates/`; source remains authoritative |
| Human-facing overview and documentation portal | `README.md` and `docs/` |
| Root license and third-party attribution | `LICENSE`, `NOTICE.md`, and `licenses/` |
| Configured root verification commands | This guide's Command Inventory; derive feature coverage from root and pinned leaf manifests |

The root package is currently the sole member of the root Cargo workspace.
Production leaves remain path dependencies, and all configured leaf paths are
explicitly excluded from workspace membership. `surgeist-test` remains a pinned
API-audit input without a production dependency or facade reexport. Resolve these
facts from `Cargo.toml` and `src/lib.rs` whenever the integration changes.

When sources disagree, report exact paths and revisions, including the root
gitlink and leaf checkout when they differ. Do not guess, silently rewrite another
authority, or widen the task to reconcile them.

## Product Boundary

Surgeist is a reusable, host-adapter-agnostic Rust UI framework built from strict,
typed, composable primitives. Public APIs, internal models, errors, defaults,
features, tests, docs, and examples are product contracts. Expose intentional
front doors and keep symbolic values unresolved until their owning layer has the
required context.

The following are intended architecture boundaries. The pinned manifests and
source establish current implementation facts and the actual dependency graph.

| Crate | Owns |
| --- | --- |
| Root `surgeist` | Thin facade, public composition, cross-crate adapters, and integration |
| `surgeist-animation` | CSS animation and transition timing, easing, keyframes, interpolation, and sampled values |
| `surgeist-css` | CSS syntax, authored values, browser recovery diagnostics, and optional strict validation |
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

- Add or repurpose a crate only for a durable API and ownership boundary. Update
  this table in the same authorized architecture change.
- Keep production dependencies directional and acyclic. A proposed dependency
  edge must preserve the owning roles above.
- Root composes leaf front-door APIs. Surgeist-to-Surgeist lowering belongs in
  root or a root-owned tool.
- Leaf internals are private by default. Do not reach through sibling private
  modules or duplicate cross-crate interpretation.
- A leaf owns backend-local adapters only when that backend belongs to its domain.
- `surgeist-test` may depend on production leaves for shared verification, but the
  root facade must not production-depend on `surgeist-test`.
- A small change requiring many leaf edits is evidence to revisit the boundary,
  not permission to create a dependency sink.

For work involving another repository, resolve that repository's ownership from
its current committed policy and source. Inspection does not grant write
authority there.

## Generated Artifacts

Root owns the only API generator and all generated API audits. Generator source
lives in `api/generator/`; generated outputs are `api/public-api.txt` and
`api/crates/*.txt`. Leaf repositories must not carry copies. Source is
authoritative; never hand-edit generated artifacts.

The generator is a separate Cargo workspace. Its target discovery scans
`crates/` for `surgeist-*` directories containing manifests, independently of root
Cargo workspace membership. Inspect `api/generator/src/lib.rs` for the actual
target selection and rustdoc toolchain configuration. An API check builds the
audit inputs and compares generated text without rewriting audit files.

Refresh leaf artifacts only after the owning leaf source is committed, published,
and visible at the root gitlink. Commit generated deltas in root with the
authorized integration change that exposed their source.

Configured generator commands are:

```sh
cargo run --manifest-path api/generator/Cargo.toml
cargo run --manifest-path api/generator/Cargo.toml -- --list
cargo run --manifest-path api/generator/Cargo.toml -- --check
cargo run --manifest-path api/generator/Cargo.toml -- --root
```

To refresh one leaf, use `--crate` with its target name from `--list`. For example:

```sh
cargo run --manifest-path api/generator/Cargo.toml -- --crate surgeist-task
```

API refresh is command-only and is not part of normal `cargo test` runs.

## Command Inventory

These commands describe current local verification capability. Assigned scope,
repository facts, and PISCT skill guidance select the applicable checks and
feature coverage. Use `$pisct:process` for authorized noninteractive checks with
already-present tooling; this inventory does not authorize software acquisition.

```sh
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -F unsafe-code -D warnings
cargo fmt --check
cargo test --manifest-path api/generator/Cargo.toml
cargo run --manifest-path api/generator/Cargo.toml -- --check
```

Root `--workspace` checks select the root package and compile its production path
dependencies; they do not run the excluded leaves' focused test suites or the
separate generator's tests. Derive leaf checks from the pinned owning repository.
Derive feature and platform checks from root and pinned leaf manifests and their
documented contracts; do not assume `--all-features` is a supported combination.

Discovery is complete when repository ownership, the product boundary, pinned
revisions, public entry points, dependency and feature facts, generated-artifact
policy, verification sources, and applicable commands are established from
current source.
