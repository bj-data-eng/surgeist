# Surgeist App Runtime Foundation 09: UiSurface Lifecycle, Root Replacement, And Isolation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add UI surface lifecycle, root replacement, and per-window isolation.

**Architecture:** This is split 09 of the numbered app-runtime foundation sequence. It should be implemented only after the earlier numbered splits are complete and reviewed, and it should preserve the typed app/work-plane boundary established by the sequence.

**Tech Stack:** Rust, Surgeist root facade crate, existing Surgeist sibling crates, crate-local unit and integration tests, fake runtime/test harnesses, and optional Tokio support only where a later split explicitly enables it.

---

## Review Gate

After implementing and committing this numbered split, stop and request a separate clean review before proceeding to the next numbered plan. The coordinator must reconcile reviewer findings and require follow-up commits when needed. Do not batch multiple numbered splits into one worker assignment unless the user explicitly approves that shortcut.

### Task 9: UiSurface Lifecycle, Root Replacement, And Isolation

**Files:**
- Create: `src/app/surface.rs`
- Modify: `src/app/mod.rs`
- Modify: `src/app/effect.rs`
- Modify: `src/app/tests.rs`

- [ ] **Step 1: Write lifecycle tests**

Add:

```rust
#[test]
fn ui_surface_lifecycle_tracks_native_state() {
    let mut surface = UiSurface::new(
        SurfaceId::from_u64(1),
        surgeist::window::Id::from_u64(10),
        WindowRoot::new(RootId::new("main")),
    );

    assert_eq!(surface.lifecycle(), SurfaceLifecycle::Created);
    surface.ready();
    assert_eq!(surface.lifecycle(), SurfaceLifecycle::Ready);
    surface.resized(surgeist::window::size(640, 480));
    assert_eq!(surface.viewport().width, 640.0);
    surface.hidden();
    assert_eq!(surface.lifecycle(), SurfaceLifecycle::Hidden);
    surface.suspended();
    assert_eq!(surface.lifecycle(), SurfaceLifecycle::Suspended);
    surface.closing();
    assert_eq!(surface.lifecycle(), SurfaceLifecycle::Closing);
    surface.closed();
    assert_eq!(surface.lifecycle(), SurfaceLifecycle::Closed);
    surface.destroyed();
    assert_eq!(surface.lifecycle(), SurfaceLifecycle::Destroyed);
}
```

- [ ] **Step 2: Write isolation tests**

Add:

```rust
#[test]
fn replacing_root_creates_distinct_retained_model() {
    let window_id = surgeist::window::Id::from_u64(20);
    let mut surface = UiSurface::new(
        SurfaceId::from_u64(1),
        window_id,
        WindowRoot::new(RootId::new("a")),
    );
    let old_retained_root = surface.retained().root();

    surface.replace_root(WindowRoot::new(RootId::new("b")));

    assert_eq!(surface.window_id(), window_id);
    assert_eq!(surface.root().id(), &RootId::new("b"));
    assert_ne!(surface.retained().root(), old_retained_root);
    assert!(surface.invalidations().contains(SurfaceInvalidation::RootReplaced));
}

#[test]
fn separate_surfaces_do_not_share_retained_or_invalidations() {
    let mut one = UiSurface::new(
        SurfaceId::from_u64(1),
        surgeist::window::Id::from_u64(1),
        WindowRoot::new(RootId::new("main")),
    );
    let two = UiSurface::new(
        SurfaceId::from_u64(2),
        surgeist::window::Id::from_u64(2),
        WindowRoot::new(RootId::new("main")),
    );

    one.invalidate(SurfaceInvalidation::SnapshotChanged(StateVersion::from_u64(2)));

    assert_ne!(one.retained().root(), two.retained().root());
    assert_eq!(one.invalidations().len(), 1);
    assert!(two.invalidations().is_empty());
}
```

- [ ] **Step 3: Run failing tests**

Run:

```sh
cargo test -p surgeist app::tests
```

Expected: fail with missing surface types.

- [ ] **Step 4: Implement surface state**

Add `surface.rs` with:

- `WindowRoot` containing `RootId` and root `retained::Element` factory input for this slice;
- `UiSurface` fields for surface id, window id, root, retained model, lifecycle, viewport, invalidations, last rendered state version, hover/focus/native focus facts, and scroll offset;
- `SurfaceLifecycle::{Created, Ready, Resized, Hidden, Occluded, Suspended, Closing, Closed, Destroyed}`;
- `SurfaceInvalidation::{RootReplaced, SnapshotChanged(StateVersion), ViewportChanged, RetainedChanged}`;
- lifecycle methods from tests;
- accessors `id`, `window_id`, `root`, `retained`, `lifecycle`, `viewport`, and `invalidations`;
- `replace_root` creates a fresh `retained::Model::empty()` or root factory model and invalidates only this surface.

- [ ] **Step 5: Verify**

Run:

```sh
cargo test -p surgeist app::tests
cargo fmt
rg -n -F -e '#[allow' -e '#[expect' -e 'allow(' src/app/surface.rs src/app/effect.rs src/app/tests.rs
```

Expected: tests pass, formatting succeeds, and the lint-suppression scan prints no matches.

- [ ] **Step 6: Commit**

```sh
git add src/app/mod.rs src/app/surface.rs src/app/effect.rs src/app/tests.rs
git commit -m "Add app UI surface lifecycle"
```

---

