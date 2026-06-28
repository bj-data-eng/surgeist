# Surgeist App Runtime Foundation 13: Headless Runtime Harness And Fakes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add headless runtime fakes and test harness support.

**Architecture:** This is split 13 of the numbered app-runtime foundation sequence. It should be implemented only after the earlier numbered splits are complete and reviewed, and it should preserve the typed app/work-plane boundary established by the sequence.

**Tech Stack:** Rust, Surgeist root facade crate, existing Surgeist sibling crates, crate-local unit and integration tests, fake runtime/test harnesses, and optional Tokio support only where a later split explicitly enables it.

---

## Review Gate

After implementing and committing this numbered split, stop and request a separate clean review before proceeding to the next numbered plan. The coordinator must reconcile reviewer findings and require follow-up commits when needed. Do not batch multiple numbered splits into one worker assignment unless the user explicitly approves that shortcut.

### Task 13: Headless Runtime Harness And Fakes

**Files:**
- Modify: `src/app/testing.rs`
- Modify: `src/app/mod.rs`
- Modify: `src/app/tests.rs`
- Modify: `tests/app.rs`

- [ ] **Step 1: Write headless harness integration test**

Add to `tests/app.rs`:

```rust
use surgeist::app::testing::HeadlessApp;

#[test]
fn headless_app_runs_without_winit_or_tokio() {
    let mut app = HeadlessApp::counter();

    app.open_surface("main");
    app.input_increment();
    app.drain();

    assert_eq!(app.counter(), 1);
    assert_eq!(app.fake_window().redraws(), &[app.surface_id("main")]);
    assert_eq!(app.fake_executor().spawned().len(), 0);
}
```

- [ ] **Step 2: Write fake clock test**

Add to `src/app/tests.rs`:

```rust
#[test]
fn fake_clock_advances_scheduled_effects_deterministically() {
    let mut harness = HeadlessHarness::counter();
    harness.schedule_timer("debounce", Duration::from_millis(50));

    assert!(harness.due_timers().is_empty());
    harness.clock_mut().advance(Duration::from_millis(50));

    assert_eq!(harness.due_timers(), vec!["debounce"]);
}
```

- [ ] **Step 3: Run failing tests**

Run:

```sh
cargo test --package surgeist --test app headless_app_runs_without_winit_or_tokio
cargo test -p surgeist app::tests
```

Expected: fail with missing testing harness types.

- [ ] **Step 4: Implement test harness**

Extend the `testing.rs` file created in Task 11 with:

- `FakeClock` with deterministic `now`, `advance`, and scheduled timer drain;
- `FakeWindowBridge` recording redraw requests and native window commands;
- reuse the existing `FakeWakeBridge` from Task 11 without redefining it;
- `HeadlessHarness<State, Reducer, Input>` wrapping runtime, fake executor, fake window bridge, and fake clock;
- `HeadlessApp::counter()` convenience fixture used by public integration tests;
- fixture input methods that enqueue app inputs instead of calling reducer directly.

Re-export `testing` as `pub mod testing;` so integration tests can use it.

- [ ] **Step 5: Verify**

Run:

```sh
cargo test --package surgeist --test app headless_app_runs_without_winit_or_tokio
cargo test -p surgeist app::tests
cargo fmt
rg -n -F -e '#[allow' -e '#[expect' -e 'allow(' src/app/testing.rs src/app/tests.rs tests/app.rs
```

Expected: tests pass, formatting succeeds, and the lint-suppression scan prints no matches.

- [ ] **Step 6: Commit**

```sh
git add src/app/mod.rs src/app/testing.rs src/app/tests.rs tests/app.rs
git commit -m "Add headless app test harness"
```

---
