# Surgeist App Runtime Foundation 07: Coordination, Scopes, Subscriptions, And Observer Policy Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add app coordination, scopes, subscriptions, observers, and task/resource/service state projection.

**Architecture:** This is split 07 of the numbered app-runtime foundation sequence. It should be implemented only after the earlier numbered splits are complete and reviewed, and it should preserve the typed app/work-plane boundary established by the sequence.

**Tech Stack:** Rust, Surgeist root facade crate, existing Surgeist sibling crates, crate-local unit and integration tests, fake runtime/test harnesses, and optional Tokio support only where a later split explicitly enables it.

---

## Review Gate

After implementing and committing this numbered split, stop and request a separate clean review before proceeding to the next numbered plan. The coordinator must reconcile reviewer findings and require follow-up commits when needed. Do not batch multiple numbered splits into one worker assignment unless the user explicitly approves that shortcut.

### Task 7: Coordination, Scopes, Subscriptions, And Observer Policy

**Files:**
- Modify: `src/app/coord.rs`
- Modify: `src/app/mod.rs`
- Modify: `src/app/tests.rs`

- [ ] **Step 1: Write scope and subscription tests**

Add:

```rust
#[test]
fn app_scope_covers_runtime_ownership_kinds() {
    assert!(AppScope::app().is_app());
    assert_eq!(
        AppScope::window(surgeist::window::Id::from_u64(9)).window_id(),
        Some(surgeist::window::Id::from_u64(9))
    );
    assert_eq!(AppScope::surface(SurfaceId::from_u64(3)).surface_id(), Some(SurfaceId::from_u64(3)));
    assert_eq!(
        AppScope::resource(ResourceId::new("graph")).resource_id(),
        Some(ResourceId::new("graph"))
    );
    assert_eq!(AppScope::custom("workspace:alpha").segments()[0].namespace(), "custom");
    assert_eq!(
        AppScope::workspace("alpha")
            .then(ScopePathSegment::new("resource", "graph"))
            .segments()
            .len(),
        2
    );
}

#[test]
fn subscriptions_attach_and_detach_observers_without_owning_work() {
    let mut coord = CoordinationState::default();
    let sub = Subscription::task(TaskKey::new("compile:main"))
        .scope(AppScope::resource(ResourceId::new("project:main")))
        .observer(SurfaceId::from_u64(1));

    coord.subscribe(sub.clone());
    assert_eq!(coord.observer_count(&sub.target()), 1);
    assert!(coord.is_observed(&sub.target()));

    coord.unsubscribe(&sub);
    assert_eq!(coord.observer_count(&sub.target()), 0);
    assert!(!coord.is_observed(&sub.target()));
}
```

- [ ] **Step 2: Write coalescing tests**

Add:

```rust
#[test]
fn coordination_coalesces_progress_by_key() {
    let mut coord = CoordinationState::default();

    coord.record_progress(ProgressEvent::new(
        TaskId::from_u64(1),
        TaskAttemptId::from_u64(1),
        CoalescingKey::new("bytes"),
        "10",
    ));
    coord.record_progress(ProgressEvent::new(
        TaskId::from_u64(1),
        TaskAttemptId::from_u64(1),
        CoalescingKey::new("bytes"),
        "20",
    ));

    let drained = coord.drain_progress_budgeted(8);
    assert_eq!(drained.len(), 1);
    assert_eq!(drained[0].payload(), "20");
    assert_eq!(coord.coalesced_progress_count(), 1);
}
```

- [ ] **Step 3: Run failing tests**

Run:

```sh
cargo test -p surgeist app::tests
```

Expected: fail with missing subscription, observer, and progress-coalescing types.

- [ ] **Step 4: Implement coordination**

Extend `coord.rs` with:

- `SubscriptionTargetKindId` as an open string-backed id with built-in constants `TASK`, `RESOURCE`, and `SERVICE`;
- `SubscriptionTarget { kind: SubscriptionTargetKindId, key: String }` with typed constructors `task(TaskKey)`, `resource(ResourceId)`, and `service(ServiceId)`;
- `Subscription` with target descriptor, scope, observer surface id, and priority;
- `CoordinationState` with observer sets by target and coalesced progress map;
- `CoalescingKey` and `ProgressEvent`;
- observer attach/detach methods and budgeted progress draining.

- [ ] **Step 5: Verify**

Run:

```sh
cargo test -p surgeist app::tests
cargo fmt
rg -n -F -e '#[allow' -e '#[expect' -e 'allow(' src/app/coord.rs src/app/tests.rs
```

Expected: tests pass, formatting succeeds, and the lint-suppression scan prints no matches.

- [ ] **Step 6: Commit**

```sh
git add src/app/mod.rs src/app/coord.rs src/app/tests.rs
git commit -m "Add app coordination scopes and subscriptions"
```

---

