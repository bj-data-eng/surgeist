# Surgeist Repository Guide

Use `$pisct:coordination` for delivery coordination and the task-appropriate
focused PISCT skill. Follow higher-priority instructions. This guide grants no
mutation, installation, commit, publication, or cross-repository authority.

## Authority Split

This file owns repository discovery, product boundaries and command inventory.
Crate-local `AGENTS.md` files supplement it with domain facts and focused checks;
they do not establish separate ownership or delivery workflows.
Keep guides as concise discovery maps. Detailed contracts belong with their
owning documentation/source; load only material relevant to the current task.

Use [surgeist-admin](.agents/skills/surgeist-admin/SKILL.md) for issues and labels,
pull requests, local working material and pause/resume handoffs. PISCT owns
engineering methods. Apply `$pisct:invariants` to engineering and
`$pisct:rust-modeling` to Rust models and APIs. Surgeist-owned Rust prohibits
executable `unsafe`.

The coordinator owns issue and label updates, staging, commits, integration and
publication within the authorized scope. Use `$pisct:holistic-review` for final
independent integration review.

## Repository Identity And Ownership

This repository owns the `surgeist` facade in `src/`, all implementations under
`crates/`, their contracts, manifests, tests, fixtures, documentation and shared
tooling. Root owns workspace wiring, cross-crate composition and API auditing.
Each consuming subsystem owns its direct integration with lower-level crates.
Original source checkouts are historical origins, outside implementation scope;
they impose no external publication or promotion prerequisite.

## Discover The Current Structure

Read the relevant sources below; manifests establish package/dependency facts
and current source establishes behavior.

| Fact | Authoritative source |
| --- | --- |
| Root identity, MSRV, features, facade dependencies, workspace/default members | [Cargo.toml](Cargo.toml) |
| Product dependency resolution | [Cargo.lock](Cargo.lock) |
| Maintained shaping source and provenance | [crates/surgeist-shaper/PROVENANCE.md](crates/surgeist-shaper/PROVENANCE.md) |
| Crate identity, dependencies, features and compatibility | Each `crates/surgeist-*/Cargo.toml` |
| Domain contracts and focused checks | Affected crate's `AGENTS.md`, `src/lib.rs`, README, tests and tracked scripts |
| Facade and root package tests | [src/lib.rs](src/lib.rs), [tests/](tests/) |
| Architecture and integration boundaries | [docs/explanation.md](docs/explanation.md), crate documentation |
| Documentation navigation and source locations | [README.md](README.md), [docs/reference.md](docs/reference.md) |
| Standards and implementation provenance | [references/SOURCE-EDITIONS.md](references/SOURCE-EDITIONS.md) |
| Licensing and bundled-material provenance | [LICENSE](LICENSE), [NOTICE.md](NOTICE.md), [licenses/](licenses/), crate-local notices |

When sources disagree, report exact paths and revisions. Do not guess, silently
rewrite another authority, or widen the task to reconcile them.

## Product Boundary

Surgeist is a reusable, host-adapter-agnostic Rust UI framework built from typed,
composable primitives. Follow the [architecture vision](https://chatgpt.com/space/page_6ce383176ca08191b1dd6d5d16e63979)
and [crate-boundary explanation](docs/explanation.md): dependencies remain
acyclic, each semantic phase has one owner, and consumers depend directly on
the contracts they use. Lower layers never depend on the facade or Runtime.
Runtime owns orchestration and coherent publication; the facade is optional
composition, not a mandatory integration intermediary.

DOM is the sole canonical live tree; HTML construction and parser reports are
immutable values, not another live DOM. HTML owns language interpretation;
consumers own their adapters. HTML behavior follows the pinned standard;
html5ever revisions identify implementation provenance, not a Surgeist dialect.
Crate READMEs and source state current capability limits.

`surgeist-test` may depend on production crates; the facade must not
production-depend on it, corpus-generation tooling or `surgeist-fuzz`.
Fuzz owns shared engines/process/evidence tooling, while consumers own cases,
codecs, expectations, seeds and regressions; benchmarking is outside its scope.
Generator owns shared CSS/browser-corpus mechanics; consumers retain semantic
conversion and corpus adapters.

The former retained owner and style implementation are removed. Planned
selectors, cascade and rebuilt style retain their separate matching, precedence
and computed-style responsibilities. Implement HTML-owned work, then DOM, then
downstream CSS crates; availability of both producers governs integration,
without blanket scheduling dependencies.

## Backward Compatibility

Before the root `surgeist` public API reaches v1.x.x, do not add or retain
compatibility/migration shims or machinery in any crate. Reaching v1 permits
consideration, not automatic authorization. CSS-standard legacy syntax/aliases
express CSS semantics and are not crate compatibility shims. Apply
`$pisct:invariants` for semantic API naming.

## Subagent Models

Use `gpt-6.1-sol` for every subagent and the coordinator, with no automatic
alternatives. Follow `$pisct:coordination` for explicit launch settings,
assignment-specific reasoning effort and review independence.

## Generated Artifacts

Source remains authoritative; never hand-edit generated artifacts.

| Artifact | Owner and procedures |
| --- | --- |
| `api/public-api.txt`, `api/crates/*.txt` | [api/generator/](api/generator/); [audit procedures](docs/how-to.md#check-public-api-audits) |
| Checked-in CSSOM property accessors | [tools/cssom-reflections/](tools/cssom-reflections/README.md); CSS owns property metadata |
| CSS/browser corpora | [surgeist-generator](crates/surgeist-generator/README.md) and each consuming crate's corpus guide |

API generation scans `crates/surgeist-*` manifests independently of workspace
membership, including support tooling. Its source owns the pinned rustdoc
toolchain and profiles. Keep it at root; no external publication/pointer update
is required. Reflection reproducibility and API freshness are separate checks.
API auditing is explicit maintenance, separate from normal product tests.

## Command Inventory

`default-members = ["."]` limits unqualified Cargo commands to the facade.
Product packages share the root lockfile; API generation and the optional
[layout Dylint catalog](crates/surgeist-layout/tools/surgeist-layout-audits/README.md)
are separate workspaces. Dylint is opt-in, not a standing product gate.

Select focused checks through `$pisct:testing` and the affected crate guide;
execute through `$pisct:process`. No blanket workspace tests. Targeted Cargo
commands may run in parallel, with one build job per command. Monitor
`pisct host status` and follow coordination's pressure guidance. Use separate
targets for concurrent worktrees changing the same packages; reuse an existing
target when source/context permit. Keep default test parallelism unless host
status recommends a limit.

Facade checks from the repository root:

```sh
cargo check --offline --locked -j 1 -p surgeist
cargo test --offline --locked -j 1 -p surgeist --lib
cargo clippy --offline --locked -j 1 -p surgeist --all-targets -- -F unsafe-code -D warnings
cargo fmt --all -- --check
```

API-tool tests and the conditional real-rustdoc fixture are documented in the
[audit procedures](docs/how-to.md#check-the-api-tool). Their separate workspace,
offline execution and one-job nested-build constraints remain in effect.
