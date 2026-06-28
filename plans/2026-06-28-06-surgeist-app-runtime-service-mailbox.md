# Surgeist App Runtime Foundation 06: Service Registry And Mailbox Policy Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed service registration and bounded mailbox policy.

**Architecture:** This split models services as typed, long-lived app capabilities with explicit command/event mailboxes and lifecycle state. A service is a contract boundary, not a requirement to spawn a daemon; implementations may later be in-process services, one-shot sidecars, app-managed local servers, external clients, or true project daemons.

**Tech Stack:** Rust, Surgeist root facade crate, existing Surgeist sibling crates, crate-local unit and integration tests, fake runtime/test harnesses, and optional Tokio support only where a later split explicitly enables it.

---

## Review Gate

After implementing and committing this numbered split, stop and request a separate clean review before proceeding to the next numbered plan. The coordinator must reconcile reviewer findings and require follow-up commits when needed. Do not batch multiple numbered splits into one worker assignment unless the user explicitly approves that shortcut.

### Task 6: Service Registry And Mailbox Policy

**Service plane policy:** Prefer in-process services and one-shot sidecars before app-managed local servers or project daemons. If a future service is backed by a child process, local server, or daemon, the service registration must still expose the same typed commands, typed events, startup/shutdown policy, failure/restart policy, mailbox limits, and diagnostics. Do not add process handles, sockets, ports, Tokio types, or daemon-specific lifecycle objects to the public app model in this split.

**Files:**
- Create: `src/app/service.rs`
- Modify: `src/app/mod.rs`
- Modify: `src/app/effect.rs`
- Modify: `src/app/tests.rs`

- [ ] **Step 1: Write service policy tests**

Add:

```rust
#[test]
fn service_registration_exposes_mailbox_policy() {
    let registration = ServiceRegistration::new(ServiceId::new("jsonrpc"))
        .scope(AppScope::app())
        .mailbox(MailboxPolicy::bounded(4).drop_oldest().observe_overflow())
        .startup(ServiceStartup::Lazy)
        .shutdown(ServiceShutdown::DrainThenStop);

    assert_eq!(registration.id(), &ServiceId::new("jsonrpc"));
    assert_eq!(registration.scope(), &AppScope::app());
    assert_eq!(registration.mailbox().capacity(), 4);
    assert_eq!(registration.mailbox().overflow(), MailboxOverflow::DropOldest);
    assert!(registration.mailbox().observe_overflow());
}

#[test]
fn service_mailbox_reports_overflow_and_keeps_capacity() {
    let policy = MailboxPolicy::bounded(2).drop_oldest().observe_overflow();
    let mut mailbox = ServiceMailbox::<u32>::new(ServiceId::new("rpc"), policy);

    mailbox.push(1);
    mailbox.push(2);
    mailbox.push(3);

    assert_eq!(mailbox.len(), 2);
    assert_eq!(mailbox.overflow_count(), 1);
    assert_eq!(mailbox.drain().collect::<Vec<_>>(), vec![2, 3]);
}
```

- [ ] **Step 2: Run failing tests**

Run:

```sh
cargo test -p surgeist app::tests
```

Expected: fail with missing service types.

- [ ] **Step 3: Implement service model**

Add `service.rs` with:

- `ServiceStatus::{Stopped, Starting, Running, Degraded, Stopping, Failed}`;
- `ServiceStartup::{Eager, Lazy}`;
- `ServiceShutdown::{Immediate, DrainThenStop}`;
- `ServiceRestart::{Never, OnFailure}`;
- `MailboxOverflow::{RejectNewest, DropNewest, DropOldest, CoalesceByKey}`;
- `MailboxPolicy` with bounded capacity, overflow behavior, observability flag, and constructors used by tests;
- `ServiceRegistration` with id, scope, mailbox, startup, shutdown, and restart policy;
- `ServiceCommandName` string-backed newtype for typed service command identity;
- `ServiceCommandPayload` opaque owned payload wrapper with `from_json_text` and `as_json_text` for the first slice;
- generic `ServiceMailbox<T>` backed by `VecDeque<T>` with push, drain, len, and overflow count.

Keep mailbox payloads owned and sendable. Service status is app-observed lifecycle truth, not proof that an operating-system process exists. Do not expose raw `String` as the service protocol; use the wrapper types even when the first payload implementation stores text internally.

- [ ] **Step 4: Add service effects**

Update `effect.rs` with:

- `EffectKindId::START_SERVICE`, `EffectKindId::STOP_SERVICE`, `EffectKindId::CALL_SERVICE`, and `EffectKindId::SERVICE_DIAGNOSTIC`;
- `StartServiceEffect { id: ServiceId }`;
- `StopServiceEffect { id: ServiceId }`;
- `CallServiceEffect { id: ServiceId, command: ServiceCommandName, payload: ServiceCommandPayload, correlation: CorrelationId }`;
- `ServiceDiagnosticEffect { id: ServiceId, diagnostic: Diagnostic }`;
- `AppEffectPayload::StartService`, `AppEffectPayload::StopService`, `AppEffectPayload::CallService`, and `AppEffectPayload::ServiceDiagnostic` variants;
- typed constructors with matching names on `AppEffect`.

Use `ServiceCommandPayload` for the first command payload lane so typed service command decoding can be layered without blocking the runtime skeleton. The service API should make that layering possible without replacing `ServiceRegistration`, mailbox policy, or lifecycle status.

Do not add a public `AppEffect::new(kind, payload)` escape hatch. The constructors must be the only public way to create these service effects so `EffectKindId` and payload cannot disagree.

- [ ] **Step 5: Verify**

Run:

```sh
cargo test -p surgeist app::tests
cargo fmt
rg -n -F -e '#[allow' -e '#[expect' -e 'allow(' src/app/service.rs src/app/effect.rs src/app/tests.rs
```

Expected: tests pass, formatting succeeds, and the lint-suppression scan prints no matches.

- [ ] **Step 6: Commit**

```sh
git add src/app/mod.rs src/app/service.rs src/app/effect.rs src/app/tests.rs
git commit -m "Add app service mailbox policy"
```

---
