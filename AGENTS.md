# Agent Guide

Use this guide for automated work in the Surgeist top-level repo.

## Product Compass

Surgeist is a reusable Rust UI framework built around strict, typed,
composable primitives for app structure, layout, style, rendering, retained
state, text, windows, dialogs, and future template/DSL layers.

Keep Surgeist host-adapter-agnostic unless a crate is explicitly about a host,
runtime, or backend. Prefer typed contracts over loose runtime behavior, hidden
shared state, or broad framework magic.

Refinement bar:

- Can this API be explained in one paragraph?
- Can this behavior be tested in its owning crate?
- Can this layer be reused outside a single app?
- Can an app override behavior without forking internals?
- Does this boundary reduce coordination, or just move complexity?

Treat public APIs, internals, tests, docs, commands, defaults, and errors as
part of the product. Names and boundaries should feel intentional.

## Repository Model

This repo is both:

- the root `surgeist` facade crate at the repo root
- the coordination workspace for sibling Surgeist crate repos linked as submodules

Expected shape:

```text
surgeist/
  Cargo.toml
  src/
  crates/
    surgeist-css/
    surgeist-dialog/
    surgeist-layout/
    surgeist-render/
    surgeist-retained/
    surgeist-shape/
    surgeist-style/
    surgeist-task/
    surgeist-test/
    surgeist-text/
    surgeist-window/
```

The root repo owns workspace wiring, submodule pointers, cross-crate plans,
source-derived API coordination, and whole-system verification. Crate
implementation work belongs in that crate's own repo and Codex project.

## Project Boundaries

Use one Codex project for this root repo and one Codex project for each crate
repo.

Top-level agents may coordinate, inspect submodules, write integration plans,
run workspace checks, update submodule pointers, and review cross-crate
compatibility.

Top-level agents must not casually edit submodule contents. If a crate needs
implementation work, do it from that crate's Codex project unless the user
explicitly asks for a top-level integration edit.

Crate project agents edit only their crate repo, run focused checks there,
commit there, and expose intentional front-door APIs for integration.

Subagents inherit the project boundary they are launched from. A root-launched
subagent does not become a crate implementation worker just because it is
assigned a crate lane. For crate implementation work, use that crate's Codex
project or thread.

Give each worker one clear repo/crate lane. Reviewers may inspect across
crates, but implementation workers should stay in their assigned project.

## Crate Roles

- root `surgeist`: thin facade, integration surface, and public composition layer.
- `surgeist-css`: strict CSS parsing and lowering into typed style data.
- `surgeist-style`: style model, resolution, and visual/layout property contracts.
- `surgeist-test`: shared test harnesses, fixture metadata, coverage and test
  quality coordination, integration tests, e2e tests, system tests, and
  integration verification support.
- `surgeist-layout`: layout algorithms, layout contracts, oracle/parity tests,
  and fixture tooling.
- `surgeist-retained`: retained identity, retained state, tree identity, and
  stable handles.
- `surgeist-text`: text shaping, measurement, font-facing abstractions, and text
  layout contracts.
- `surgeist-render`: rendering contracts and backend-facing draw data.
- `surgeist-window`: window, app host, event-loop, and platform host contracts.
- `surgeist-dialog`: dialog contracts and dialog coordination primitives.
- `surgeist-shape`: shape, geometry, and primitive path data.
- `surgeist-task`: task scheduling, work-plane contracts, cancellation,
  progress, resource-class admission, and executor-facing task policy.

Add crates only for real API boundaries, not architecture theater.

## Dependency Direction

Keep dependencies directional and acyclic. If a small change needs edits across
many crates, stop and revisit the boundary.

Current intended shape:

```text
root surgeist crate
  -> surgeist-css
  -> surgeist-dialog
  -> surgeist-layout
  -> surgeist-render
  -> surgeist-retained
  -> surgeist-shape
  -> surgeist-style
  -> surgeist-task
  -> surgeist-text
  -> surgeist-window

surgeist-css -> surgeist-style
surgeist-style -> surgeist-layout, surgeist-retained, surgeist-text
surgeist-task may depend on production crates only when task contracts require
typed integration, and should keep executor backends behind crate-owned
contracts
surgeist-render -> surgeist-shape, surgeist-window optional
surgeist-text -> surgeist-render optional
surgeist-test may depend on production crates for verification
```

Plan cross-crate API changes at the top level, then implement crate-local pieces
in the owning crate projects.

## API Coordination

API coordination is source-derived only. Do not maintain handwritten API truth
tables as authority.

Each crate should expose intentional front-door APIs from `lib.rs`, keep
internals private by default, and support generated API shape checks when that
tooling exists. The root repo may consume generated API reports, but source
remains the source of truth.

Prefer typed commands, events, snapshots, reports, and change sets. Avoid
sibling crates reaching through private module paths, broad common crates that
become dependency sinks, and accidental cycles introduced for convenience.

## Public API Surface

Treat public APIs as product contracts. Public surface includes `pub` items,
reexports, feature flags, error types, trait impls, docs, examples, and MSRV.
Generated API reports are audit artifacts for that source-derived contract, not
the source of truth.

For release-facing API changes:

- name the owning crate and public entry point
- explain whether the change is additive, breaking, or internal-only
- update docs, examples, and API artifacts when the owning crate requires them
- run source-derived API checks or `cargo semver-checks` when that tooling is
  configured

## Type And Value Modeling

Use `guidance/surgeist-rust-modeling-guide.md` when designing, reviewing, or
refactoring Rust models. Keep this repo aligned with that guide: semantic types,
explicit phases, narrow conversion boundaries, and symbolic values resolved only
by the owning layer.

## Unsafe Code

Default to no new `unsafe` code.

If `unsafe` is unavoidable:

- get explicit maintainer or user approval before adding it
- keep the unsafe block as small as possible
- document the safety contract at the unsafe boundary
- add focused tests around the invariants the unsafe code relies on
- require reviewer attention on the safety argument

Do not add broad `#[allow(...)]` attributes to quiet warnings. Prefer scoped
`#[expect(...)]` with a reason when a lint exception is intentional.

## Dependencies And Features

Prefer workspace dependencies and existing crate-local dependencies. New
dependencies must justify why they belong at that layer, whether they affect
MSRV, licenses, advisories, binary size, optional features, or dependency
cycles.

Feature checks must match the crate's real feature matrix. Do not assume
`--all-features` is valid when features are mutually exclusive, host-specific,
backend-specific, or generator-only. Document the correct feature combinations
in the owning crate guide when broad Cargo commands are misleading.

Run `cargo deny` or equivalent dependency checks when configured by the owning
repo. Do not introduce secrets, credential bypasses, network shortcuts, or CI
bypasses.

## Generated Artifacts

Generated files are owned by their generator. Do not hand-edit generated output
unless the owning crate explicitly says that is allowed.

When generated API reports, fixtures, snapshots, bindings, or parity artifacts
change:

- run the documented generator or snapshot update command
- commit source inputs and generated outputs together when the owning crate
  expects both
- explain why the generated delta is expected
- do not weaken tests or snapshots just to make a check pass

## Upstream Issue Reporting

Workers and reviewers may identify issues in sibling or upstream crates while
working from another repo. They must not edit that crate from the wrong Codex
project.

When an upstream issue affects correctness, compatibility, integration, or
developer workflow:

- Confirm the owning repo/crate.
- Capture reproduction steps, expected behavior, observed behavior, affected
  APIs/files, and relevant commands/tests/plans/commits.
- Report the issue in the owning GitHub repo.
- If GitHub issue creation is unavailable, write a complete issue draft in the
  task output and stop for coordinator action.

Issue reports should be specific enough for a crate-local worker to act without
rediscovering the problem. Do not file upstream issues for bugs owned by the
current repo; fix those locally.

## Plans And Specs

Use Superpowers for workflow guidance. Repo file locations override Superpowers
default paths.

Plans go in `/plans` at the root of the repo where the implementation will happen:

- Top-level integration plans: `surgeist/plans/`
- Crate-local plans: that crate repo's `plans/`

If writing specs or design docs, use the same root-local convention unless the
user chooses a separate folder. Do not put new plans under `docs/superpowers`.
This repo-local plan location intentionally overrides Superpowers default paths.

## Testing

Each crate owns its focused test commands. The root repo coordinates
whole-system checks, but tight iteration should happen in the relevant crate
project.

Expected command pattern:

```sh
cargo test -p <crate>
cargo clippy -p <crate> --all-targets -- -D warnings
cargo fmt --check
```

These commands are a baseline, not a substitute for crate-specific guidance.
Use the owning crate's `AGENTS.md`, `README.md`, plans, or task runner when it
defines feature-specific, generated-artifact, snapshot, fuzzing, or dependency
checks.

For root-owned code or integration wiring, run root or workspace Clippy when it
is expected to pass:

```sh
cargo clippy --workspace --all-targets -- -D warnings
```

If root Clippy is skipped because of a known upstream failure, feature matrix
constraint, or unavailable platform dependency, name the reason in the task
output.

Run focused checks before committing. Run broader root checks before updating
submodule pointers or declaring cross-crate work complete.

## Subagents

Top-level coordinators are coordinators first. For implementation work, they
assign, verify, reconcile, and integrate; they do not default to doing the
code edit themselves.

For any requested code change, the top-level coordinator must follow this
sequence unless the user explicitly waives it:

1. Check root status and relevant crate status before work begins.
2. Identify the owning repo or crate lane.
3. If executing an implementation plan, split the plan into its sequential
   tasks and assign workers one task or tightly coupled task group at a time.
   Do not hand an entire multi-task plan to one worker unless the user
   explicitly approves that shortcut.
4. Assign one implementation worker to the current scoped task in that lane.
5. Wait for the worker's result, including its reported tests and git status.
6. Assign a separate reviewer to inspect the worker's scoped changes before
   moving to the next task.
7. Reconcile the worker and reviewer findings before assigning follow-up work.
8. After all scoped tasks are complete, assign a final holistic reviewer to
   inspect the complete result against the implementation plan, crate boundary,
   tests, and git diff.
9. Confirm the owning repo has a clean committed branch. Require it to be
   pushed when another repo/thread must fetch it, when updating root submodule
   pointers, or when the user requested publication.
10. Run the relevant root-level integration checks only after crate-local work
   is complete.
11. Commit only root-owned integration changes from the top-level repo, such as
   submodule pointer updates, top-level plans, requirements, workflow docs, or
   facade wiring.
12. Push root commits when the user requested publication or when a pointer
    update must be available to other coordinators.

During plan execution, assign one clear repo/crate lane and scoped plan task to
each worker. The coordinator owns sequencing, integration, and deciding when the
next task is safe to start. Tell workers they are not alone in the codebase and
must not revert others' work.

The coordinator may directly edit root-only planning, documentation,
requirements, or workflow files when the user explicitly asks for a top-level
repo change.

Root code changes, including facade wiring, still count as code changes and
must go through the worker and reviewer gate unless the user explicitly waives
it. If implementation code must change in a crate, use that crate's Codex
project or a worker assigned from that crate project.

No coordinator may declare implementation complete until a separate reviewer
has reviewed the changed code, or the user explicitly waives review.

- Do not duplicate a completed subagent's investigation. Review, verify,
  reconcile, and act on it.
- Use clean reviewers for code changes, boundary changes, API changes, and
  nontrivial cross-crate work.
- Do not declare a multi-task implementation plan complete until the
  task-scoped worker/reviewer cycles and final holistic review are clean.

## Submodule Pointer Updates

The root repo is the known-good integration pinboard.

Do not update submodule pointers unless:

- the owning crate changes are committed and pushed
- the pinned submodule commit is fetchable from the configured submodule remote
- crate-local focused checks passed
- root `cargo check --workspace` passed
- root `cargo test --workspace` passed
- root `cargo clippy --workspace --all-targets -- -D warnings` passed, or the
  task output names why it is not expected to pass
- root `cargo fmt --check` passed
- the coordinator reviewed `git diff --submodule=log` or an equivalent
  submodule summary

Exception: if a red pointer update is explicitly requested, the root commit
message or task output must name the failing command, failing test or error,
owning issue/PR, and why the pointer update is still intentional.

Root-owned planning, documentation, workflow, requirements, and facade-only
changes may still be committed while an unrelated upstream crate issue is red,
but the task output must name the failing broad check and confirm the root
change did not attempt to hide or work around that failure.

## Editing And Git

- Use `apply_patch` for manual edits.
- Prefer `rg` and `rg --files`.
- Check status before and after edits: `git status --short --branch`.
- Before committing, review `git diff --stat` and the relevant detailed diff.
- Do not rewrite unrelated files or revert user changes unless explicitly
  asked.
- Do not create or switch branches for ordinary Surgeist work. Use the current
  `main` branch and sequential task-scoped commits unless the user explicitly
  asks for a branch or worktree.
- Keep `.venv/`, `target/`, `build/`, `dist/`, secrets, host identity, editor
  residue, and runtime residue out of git.
- Commit logical points with short, concrete messages.
- Push commits only when requested, when handing work to another repo/thread,
  or when updating root pointers to commits that other users must fetch.

Commit in the repo being changed:

- crate implementation commits inside that crate repo
- submodule pointer and integration commits inside the root Surgeist repo

Never silently edit a sibling submodule from the root project and call it done.
If that happens by mistake, stop, report it, and reconcile deliberately.
