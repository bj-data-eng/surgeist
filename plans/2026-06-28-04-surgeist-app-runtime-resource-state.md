# Surgeist App Runtime Foundation 04: Resource State Machine And Snapshot Data Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add resource identity, freshness, status, and snapshot data.

**Architecture:** This is split 04 of the numbered app-runtime foundation sequence. It should be implemented only after the earlier numbered splits are complete and reviewed, and it should preserve the typed app/work-plane boundary established by the sequence.

**Tech Stack:** Rust, Surgeist root facade crate, existing Surgeist sibling crates, crate-local unit and integration tests, fake runtime/test harnesses, and optional Tokio support only where a later split explicitly enables it.

---

## Review Gate

After implementing and committing this numbered split, stop and request a separate clean review before proceeding to the next numbered plan. The coordinator must reconcile reviewer findings and require follow-up commits when needed. Do not batch multiple numbered splits into one worker assignment unless the user explicitly approves that shortcut.

### Task 4: Resource State Machine And Snapshot Data

**Files:**
- Create: `src/app/resource.rs`
- Modify: `src/app/mod.rs`
- Modify: `src/app/effect.rs`
- Modify: `src/app/tests.rs`

- [ ] **Step 1: Write resource transition tests**

Add:

```rust
#[test]
fn resource_state_tracks_freshness_and_refreshing_independently() {
    let mut resource = ResourceState::<u32, String>::idle(ResourceId::new("thumb:1"));

    resource.starting();
    assert_eq!(resource.status(), ResourceStatus::Starting);
    assert!(!resource.is_renderable());

    resource.ready(7, Freshness::Fresh);
    assert_eq!(resource.status(), ResourceStatus::Ready);
    assert_eq!(resource.value(), Some(&7));
    assert!(resource.is_renderable());
    assert_eq!(resource.freshness(), Freshness::Fresh);

    resource.refreshing();
    assert_eq!(resource.status(), ResourceStatus::Refreshing);
    assert_eq!(resource.value(), Some(&7));
    assert!(resource.is_renderable());

    resource.mark_stale("source changed");
    assert_eq!(resource.freshness(), Freshness::Stale);
    assert_eq!(resource.stale_reason(), Some("source changed"));
}

#[test]
fn resource_failure_preserves_renderable_stale_value() {
    let mut resource = ResourceState::<u32, String>::ready(
        ResourceId::new("query:1"),
        10,
        Freshness::Fresh,
    );

    resource.refreshing();
    resource.failed("timeout".to_string(), FailureVisibility::KeepStaleValue);

    assert_eq!(resource.status(), ResourceStatus::Failed);
    assert_eq!(resource.value(), Some(&10));
    assert_eq!(resource.error(), Some(&"timeout".to_string()));
    assert!(resource.is_renderable());
}
```

- [ ] **Step 2: Run failing tests**

Run:

```sh
cargo test -p surgeist app::tests
```

Expected: fail with missing resource types.

- [ ] **Step 3: Implement resource state**

Add `resource.rs` with:

- `ResourceStatus::{Idle, Starting, Running, Refreshing, Ready, Failed, Cancelling, Cancelled, Stale}`;
- `Freshness::{Fresh, Stale}`;
- `FailureVisibility::{ClearValue, KeepStaleValue}`;
- generic `ResourceState<T, E>` storing id, status, value, error, freshness, stale reason, version, and observer count;
- transition methods used by tests;
- `ResourceSnapshot<T, E>` cloneable view for render/state binding.

- [ ] **Step 4: Connect resource effects**

Update `effect.rs` with:

- `EffectKindId::LOAD_RESOURCE` and `EffectKindId::INVALIDATE_RESOURCE`;
- `LoadResourceEffect { id: ResourceId, scope: AppScope }`;
- `InvalidateResourceEffect { id: ResourceId, reason: String }`;
- constructors `AppEffect::load_resource(id, scope)` and `AppEffect::invalidate_resource(id, reason)`.

- [ ] **Step 5: Verify**

Run:

```sh
cargo test -p surgeist app::tests
cargo fmt
rg -n -F -e '#[allow' -e '#[expect' -e 'allow(' src/app/resource.rs src/app/effect.rs src/app/tests.rs
```

Expected: tests pass, formatting succeeds, and the lint-suppression scan prints no matches.

- [ ] **Step 6: Commit**

```sh
git add src/app/mod.rs src/app/resource.rs src/app/effect.rs src/app/tests.rs
git commit -m "Add app resource state machine"
```

---

