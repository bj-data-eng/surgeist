# Surgeist App Runtime Foundation 16: Final Verification And Review Handoff Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add final verification, review handoff, and self-review checklist for the split plan sequence.

**Architecture:** This split verifies the entire numbered plan sequence as one coherent app-runtime foundation. The final review must check the code, tests, public API, task/service/executor boundaries, and separate-plane architecture rather than reviewing only the last verification diff.

**Tech Stack:** Rust, Surgeist root facade crate, existing Surgeist sibling crates, crate-local unit and integration tests, fake runtime/test harnesses, and optional Tokio support only where a later split explicitly enables it.

---

## Review Gate

After implementing and committing this numbered split, stop and request a separate clean review before proceeding to the next numbered plan. The coordinator must reconcile reviewer findings and require follow-up commits when needed. Do not batch multiple numbered splits into one worker assignment unless the user explicitly approves that shortcut.

### Task 16: Final Verification And Review Handoff

**Holistic review policy:** The final reviewer must read plans 01 through 16, the resulting git diff, and the architecture spec. They should verify that reducers remain synchronous, app state mutates only through typed inputs, work-plane output returns as task/service events, Tokio stays feature-gated, and daemon/process assumptions do not leak into public app APIs.

**Files:**
- Read-only verification across the Surgeist crate and new plan implementation.

- [ ] **Step 1: Run focused app tests**

Run:

```sh
cargo test -p surgeist app::tests
cargo test --package surgeist --test app
```

Expected: all app module unit tests and public app integration tests pass.

- [ ] **Step 2: Run feature-gated executor tests**

Run:

```sh
cargo test --package surgeist --features app-runtime-tokio app::tests::tokio_executor_is_hidden_behind_adapter
```

Expected: feature-gated test passes and no Tokio type appears in public integration test signatures.
The test should import `TokioExecutor` from `surgeist::app::runtime_tokio`, and default `cargo test -p surgeist` should not compile or export `runtime_tokio`.

- [ ] **Step 3: Verify thumbnail example contract**

Run the headless thumbnail example contract and example binary:

```sh
cargo test --package surgeist --test app thumbnail_import_example_contract_runs_headless
cargo run -p surgeist --example app-thumbnail-import
```

Expected: the contract test passes and the example prints the compact summary from Task 15.

- [ ] **Step 4: Run full Surgeist crate tests**

Run:

```sh
cargo test -p surgeist
```

Expected: all Surgeist crate tests pass.

- [ ] **Step 5: Run workspace check and Clippy**

Run:

```sh
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Expected: workspace check and Clippy pass without warnings. If a known upstream crate issue prevents workspace Clippy from passing, record the failing command, owner, and issue before completion; do not hide it with lint suppressions.

- [ ] **Step 6: Run formatting**

Run:

```sh
cargo fmt
```

Expected: formatting completes with no diff outside intentional files.

- [ ] **Step 7: Verify zero lint suppressions**

Run:

```sh
rg -n -F -e '#[allow' -e '#[expect' -e 'allow(' src/app tests/app.rs examples/app-thumbnail-import.rs
```

Expected: the scan prints no matches. The implementation must keep zero lint suppression attributes or local lint-suppression calls.

- [ ] **Step 8: Inspect status**

Run:

```sh
git status --short
```

Expected: only files changed by the app implementation tasks appear.

- [ ] **Step 9: Request final holistic review**

Ask a clean reviewer to inspect:

- final `src/app` public API against the architecture spec;
- default build without Tokio;
- feature-gated Tokio adapter isolation;
- reducer purity and no hidden side effects;
- runtime queue budgets, stale event dropping, and diagnostics;
- retained/window boundary preservation;
- fake service/prototype coverage for cancellation, backpressure, and shared observation;
- absence of daemon-specific APIs such as public process handles, ports, sockets, raw child handles, raw Tokio handles, or raw channels in app authoring APIs;
- absence of lint suppressions and generated/manual artifact drift.

Expected: reviewer returns clean or all findings are addressed by earlier-task follow-up commits before completion.

- [ ] **Step 10: Commit verification fixes if needed**

If the verification steps required small fixes inside files already touched by this plan, commit them:

```sh
git status --short
git add -p
git commit -m "Stabilize app runtime tests"
```

Expected: only reviewed hunks from files shown by `git status --short` are staged. If every changed file is wholly owned by this app-runtime implementation, explicit path staging is also acceptable, for example `git add src/app/runtime.rs src/app/tests.rs`.

## Self-Review Checklist

- Spec coverage: tasks cover the app module skeleton, IDs/provenance/diagnostics, reducer/effect contract, resource state machine, task registry/status/attempt/cancellation, service registry/mailbox policy, coordination/subscriptions/scopes, retained bridge, `UiSurface` lifecycle/isolation, runtime queues/effect execution, `AppProxy` wake bridge, executor adapter and optional Tokio feature, headless fakes, stress/prototype tests, and a small example.
- Boundary check: no task places app semantics inside `surgeist::window`; the only window integration is through descriptors, ids, commands, and the wake bridge trait.
- Core compile check: default `cargo test -p surgeist` must compile without Tokio-backed app runtime types unless feature-gated tests request `app-runtime-tokio`.
- Separate-plane check: async means command/event separation across typed app/work planes, not raw async functions in UI code or Tokio in public app APIs.
- Service/process check: plans prefer in-process services and one-shot sidecars before local servers or daemons; daemon-specific handles and lifecycle objects stay behind future adapters.
- Deferred scope check: full template syntax, real feature services, multi-root composition, advanced scheduling, and replay/audit systems are intentionally outside this first slice.
- Worker handoff: each task has specific files, tests to write first, commands with expected results, implementation scope, verification commands, and a concrete commit message.
